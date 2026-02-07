fn main() -> anyhow::Result<()> {
    println!("Strata Editor - Coming soon!");

    // For now, just run the engine like the example
    let engine = strata::Engine::new()?;
    engine.run()?;
    Ok(())
}
