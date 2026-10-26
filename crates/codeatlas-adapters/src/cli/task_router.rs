pub fn run_task_router(task: &str) -> anyhow::Result<()> {
    let lower = task.to_lowercase();
    let (model, effort) =
        if lower.contains("architect") || lower.contains("refactor") || lower.contains("migrate") {
            ("Claude Opus 4.6", "high")
        } else {
            ("Gemini 3.8 Flash", "medium")
        };
    println!("MODEL_NAME={}", model);
    println!("EFFORT={}", effort);
    Ok(())
}
