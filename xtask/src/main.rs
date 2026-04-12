use anyhow::{Context, Result, bail};
use base64::Engine;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Project automation tasks for strata-engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a Jira story from a natural-language prompt
    JiraStory {
        /// Natural language prompt describing the work
        prompt: String,
    },
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    output: Vec<OpenAiOutputItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAiOutputItem {
    content: Vec<OpenAiContentItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAiContentItem {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneratedStory {
    summary: String,
    description_markdown: String,
    acceptance_criteria: Vec<String>,
    labels: Vec<String>,
    story_points: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct JiraCreateIssueResponse {
    id: String,
    key: String,
    #[allow(dead_code)]
    self_field: Option<String>,
}

fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::JiraStory { prompt } => create_jira_story(&prompt),
    }
}

fn create_jira_story(prompt: &str) -> Result<()> {
    let openai_api_key = required_env("OPENAI_API_KEY")?;
    let jira_base_url = required_env("JIRA_BASE_URL")?;
    let jira_email = required_env("JIRA_EMAIL")?;
    let jira_api_token = required_env("JIRA_API_TOKEN")?;
    let jira_project_key = required_env("JIRA_PROJECT_KEY")?;
    let jira_issue_type =
        env::var("JIRA_ISSUE_TYPE").unwrap_or_else(|_| "Story".to_string());
    let openai_model =
        env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.4".to_string());

    let generated =
        generate_story_with_openai(&openai_api_key, &openai_model, prompt)
            .context("failed to generate story with OpenAI")?;

    let description_adf = render_jira_description(&generated);

    let created = create_issue_in_jira(
        &jira_base_url,
        &jira_email,
        &jira_api_token,
        &jira_project_key,
        &jira_issue_type,
        &generated.summary,
        &description_adf,
        &generated.labels,
    )
    .context("failed to create Jira issue")?;

    println!("Created Jira issue: {}", created.key);
    println!("Jira internal id: {}", created.id);

    Ok(())
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| {
        format!("missing required environment variable: {name}")
    })
}

fn generate_story_with_openai(
    api_key: &str,
    model: &str,
    user_prompt: &str,
) -> Result<GeneratedStory> {
    let client = reqwest::blocking::Client::new();

    let system_prompt = r#"
    You are helping generate Jira stories for a high-performance Rust game engine project called 'strata-engine'.

    The engine focuses on:
    - Vulkan rendering
    - low-level memory management (arenas, allocators)
    - ECS-style architecture
    - performance-critical systems

    Return ONLY valid JSON matching this schema:
    {
      "summary": "string",
      "description_markdown": "string",
      "acceptance_criteria": ["string"],
      "labels": ["string"],
      "story_points": 1
    }

    Guidelines:

    Summary:
    - concise, technical, and specific
    - describe the actual engineering change

    Description:
    - explain the problem and intent clearly
    - include implementation direction when useful
    - assume an experienced engine developer audience

    Acceptance Criteria:
    - must be testable and concrete
    - prefer engine-specific validation like:
      - feature works in sample/demo scene
      - no GPU validation errors introduced
      - no memory leaks or invalid accesses
      - correct behavior across multiple frames
      - no regression in performance-critical paths

    Labels:
    - ALWAYS choose from this list when applicable:
      ["rendering", "vulkan", "ecs", "scene", "asset-pipeline", "tooling", "editor", "platform", "math", "physics", "memory", "allocator"]

    - You may include multiple labels
    - Prefer specific labels over generic ones

    Style:
    - avoid generic product management language
    - avoid fluff
    - write like an engine programmer

    Do NOT include any text outside the JSON.
    Do NOT use markdown code fences.
    "#;

    let payload = json!({
        "model": model,
        "input": [
            {
                "role": "system",
                "content": [
                    { "type": "input_text", "text": system_prompt }
                ]
            },
            {
                "role": "user",
                "content": [
                    { "type": "input_text", "text": format!("Create a Jira story from this prompt: {user_prompt}") }
                ]
            }
        ]
    });

    let response = client
        .post("https://api.openai.com/v1/responses")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()?
        .error_for_status()?
        .json::<OpenAiResponse>()?;

    let text = extract_text(response)?;
    let story: GeneratedStory =
        serde_json::from_str(&text).with_context(|| {
            format!("OpenAI returned non-JSON or unexpected JSON:\n{text}")
        })?;

    Ok(story)
}

fn extract_text(response: OpenAiResponse) -> Result<String> {
    for item in response.output {
        for content in item.content {
            if content.kind == "output_text" {
                if let Some(text) = content.text {
                    return Ok(text);
                }
            }
        }
    }

    bail!("could not find output_text in OpenAI response")
}

/// Convert a plain-text/markdown description into Atlassian Document Format (ADF).
/// Handles headings (##), bullet points (-), and plain paragraphs.
fn markdown_to_adf(text: &str) -> serde_json::Value {
    let mut content: Vec<serde_json::Value> = Vec::new();

    let mut bullet_items: Vec<serde_json::Value> = Vec::new();

    let flush_bullets =
        |items: &mut Vec<serde_json::Value>,
         content: &mut Vec<serde_json::Value>| {
            if !items.is_empty() {
                content.push(json!({
                    "type": "bulletList",
                    "content": items.drain(..).collect::<Vec<_>>()
                }));
            }
        };

    for line in text.lines() {
        let line = line.trim();

        if let Some(heading) = line.strip_prefix("## ") {
            flush_bullets(&mut bullet_items, &mut content);
            content.push(json!({
                "type": "heading",
                "attrs": { "level": 2 },
                "content": [{ "type": "text", "text": heading }]
            }));
        } else if let Some(item) = line.strip_prefix("- ") {
            bullet_items.push(json!({
                "type": "listItem",
                "content": [{
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": item }]
                }]
            }));
        } else if line.is_empty() {
            flush_bullets(&mut bullet_items, &mut content);
        } else {
            flush_bullets(&mut bullet_items, &mut content);
            content.push(json!({
                "type": "paragraph",
                "content": [{ "type": "text", "text": line }]
            }));
        }
    }

    flush_bullets(&mut bullet_items, &mut content);

    json!({ "type": "doc", "version": 1, "content": content })
}

fn render_jira_description(story: &GeneratedStory) -> serde_json::Value {
    let mut text = String::new();

    text.push_str(story.description_markdown.trim());
    text.push_str("\n\n## Acceptance Criteria\n");

    for criterion in &story.acceptance_criteria {
        text.push_str("- ");
        text.push_str(criterion.trim());
        text.push('\n');
    }

    if let Some(points) = story.story_points {
        text.push_str("\n## Estimation\n");
        text.push_str(&format!("- Suggested story points: {points}\n"));
    }

    markdown_to_adf(&text)
}

fn create_issue_in_jira(
    base_url: &str,
    email: &str,
    api_token: &str,
    project_key: &str,
    issue_type: &str,
    summary: &str,
    description: &serde_json::Value,
    labels: &[String],
) -> Result<JiraCreateIssueResponse> {
    let client = reqwest::blocking::Client::new();

    let auth = base64::engine::general_purpose::STANDARD
        .encode(format!("{email}:{api_token}"));

    let payload = json!({
        "fields": {
            "project": { "key": project_key },
            "summary": summary,
            "description": description,
            "issuetype": { "name": issue_type },
            "labels": labels
        }
    });

    let response = client
        .post(format!("{base_url}/rest/api/3/issue"))
        .header(AUTHORIZATION, format!("Basic {auth}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        bail!("Jira returned {status}: {body}");
    }

    let mut created = response.json::<JiraCreateIssueResponse>()?;
    created.self_field = None;
    Ok(created)
}
