//! Integration tests for database-backed prompt system
//! Tests that prompts are loaded from database and not from environment variables

use mimivibe_backend::repository::PromptRepository;
use sqlx::PgPool;
use std::env;

#[sqlx::test]
async fn test_database_contains_all_required_prompts(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations first
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Load all active prompts
    let prompts = PromptRepository::load_all_active_prompts(&pool).await?;

    // Verify we have exactly 3 prompts
    assert_eq!(prompts.len(), 3, "Expected exactly 3 prompts in database");

    // Verify all required agent names exist
    let agent_names: Vec<String> = prompts.iter().map(|p| p.agent_name.clone()).collect();
    assert!(agent_names.contains(&"question_filter".to_string()));
    assert!(agent_names.contains(&"question_analyzer".to_string()));
    assert!(agent_names.contains(&"reading_agent".to_string()));

    // Verify all prompts are active
    for prompt in &prompts {
        assert!(
            prompt.is_active,
            "Prompt {} should be active",
            prompt.agent_name
        );
        assert!(
            !prompt.prompt_content.is_empty(),
            "Prompt content should not be empty"
        );
        assert!(prompt.version >= 1, "Prompt version should be >= 1");
    }

    Ok(())
}

#[sqlx::test]
async fn test_load_individual_prompts_from_database(
    pool: PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations first
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Test loading each individual prompt
    let question_filter = PromptRepository::load_prompt(&pool, "question_filter").await?;
    assert_eq!(question_filter.agent_name, "question_filter");
    assert!(question_filter
        .prompt_content
        .contains("คุณคือผู้ช่วยคัดกรองคำถาม"));

    let question_analyzer = PromptRepository::load_prompt(&pool, "question_analyzer").await?;
    assert_eq!(question_analyzer.agent_name, "question_analyzer");
    assert!(question_analyzer
        .prompt_content
        .contains("คุณคือผู้ช่วยวิเคราะห์คำถาม"));

    let reading_agent = PromptRepository::load_prompt(&pool, "reading_agent").await?;
    assert_eq!(reading_agent.agent_name, "reading_agent");
    assert!(reading_agent.prompt_content.contains("แม่หมอมีมี่"));

    Ok(())
}

#[sqlx::test]
async fn test_database_prompt_performance(pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations first
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Measure query performance for loading all prompts
    let start = std::time::Instant::now();
    let _prompts = PromptRepository::load_all_active_prompts(&pool).await?;
    let duration = start.elapsed();

    // Query should complete in less than 100ms (as per requirements)
    assert!(
        duration.as_millis() < 100,
        "Prompt loading took too long: {}ms",
        duration.as_millis()
    );

    // Measure individual prompt loading performance
    let start = std::time::Instant::now();
    let _prompt = PromptRepository::load_prompt(&pool, "question_filter").await?;
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 50,
        "Individual prompt loading took too long: {}ms",
        duration.as_millis()
    );

    Ok(())
}

#[test]
fn test_env_has_no_prompt_entries() {
    // Verify that environment variables for prompts are NOT set
    // These should be empty or not exist
    let env_vars = [
        "QUESTION_FILTER_PROMPT",
        "QUESTION_ANALYSIS_PROMPT",
        "READING_AGENT_PROMPT",
        "PROMPT_QUESTION_FILTER",
        "PROMPT_QUESTION_ANALYSIS",
        "PROMPT_READING_AGENT",
    ];

    for var in env_vars.iter() {
        let value = env::var(var).unwrap_or_default();
        assert!(
            value.is_empty(),
            "Environment variable {} should not be set or should be empty",
            var
        );
    }
}

#[test]
fn test_env_example_contains_no_prompt_entries() {
    // Read .env.example file
    let env_example =
        std::fs::read_to_string(".env.example").expect("Should be able to read .env.example file");

    // Check that it doesn't contain any prompt-related environment variables
    let forbidden_patterns = [
        "QUESTION_FILTER_PROMPT",
        "QUESTION_ANALYSIS_PROMPT",
        "READING_AGENT_PROMPT",
        "PROMPT_QUESTION_FILTER",
        "PROMPT_QUESTION_ANALYSIS",
        "PROMPT_READING_AGENT",
    ];

    for pattern in forbidden_patterns.iter() {
        assert!(
            !env_example.contains(pattern),
            ".env.example should not contain {}",
            pattern
        );
    }

    // But it should still contain DATABASE_URL
    assert!(
        env_example.contains("DATABASE_URL"),
        ".env.example should contain DATABASE_URL example"
    );
}
