fn main() -> anyhow::Result<()> {
    let engine = strata::Engine::new()?;
    engine.run()?;
    Ok(())
}
