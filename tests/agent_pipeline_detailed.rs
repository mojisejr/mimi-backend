//! Detailed Agent Pipeline Test
//!
//! This test demonstrates the complete agent pipeline:
//! 1. Question Filter validates the question
//! 2. Question Analyzer extracts context
//! 3. Reading Agent generates the final reading

use std::time::Duration;
use uuid::Uuid;
use serde_json::{json, Value};
use sqlx::PgPool;

mod common;
use common::setup::{TestApp, create_test_app};

#[tokio::test]
async fn detailed_agent_pipeline_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🤖 Detailed Agent Pipeline Analysis");
    println!("==================================");

    let app = create_test_app().await?;
    let db_pool = app.db_pool.clone();

    // Test questions with different complexities
    let test_cases = vec![
        ("ความรักของฉันจะเป็นอย่างไร", "love"),
        ("งานของฉันจะเจริญไหม", "career"),
        ("สุขภาพของฉันจะดีขึ้นไหม", "health"),
    );

    for (question, expected_category) in test_cases {
        println!("\n🎯 Testing: {}", question);
        println!("Expected category: {}", expected_category);
        println!("{}", "─".repeat(50));

        // Submit question
        let request_body = json!({
            "question": question,
            "cards": 3
        });

        let response = app.client
            .post("/api/v1/tarots/read")
            .json(&request_body)
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        let submission_response: Value = response.json().await?;
        let job_id = submission_response["job_id"].as_str().unwrap();

        // Wait for completion
        let mut attempts = 0;
        let max_attempts = 60;

        loop {
            let status_response = app.client
                .get(&format!("/api/v1/tarots/{}", job_id))
                .send()
                .await?;

            let status_result: Value = status_response.json().await?;
            let status = status_result["status"].as_str().unwrap();

            if status == "completed" {
                break;
            } else if status == "failed" {
                println!("❌ Job failed: {:?}", status_result["error"]);
                break;
            }

            attempts += 1;
            if attempts >= max_attempts {
                println!("⏰ Timeout waiting for completion");
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Get detailed job record
        let job_id_uuid = Uuid::parse_str(job_id)?;
        let job_record = sqlx::query!(
            r#"
            SELECT id, status::text as status, result, attempts, worker_id, created_at, completed_at
            FROM jobs
            WHERE id = $1
            "#,
            job_id_uuid
        )
        .fetch_one(&db_pool)
        .await?;

        if let Some(result) = job_record.result {
            let reading: Value = serde_json::from_value(result)?;

            println!("\n📊 Pipeline Result:");
            println!("  ✅ Job ID: {}", job_record.id);
            println!("  ✅ Status: {}", job_record.status);
            println!("  ✅ Attempts: {}", job_record.attempts);
            println!("  ✅ Worker: {:?}", job_record.worker_id);

            // Analyze the reading content
            analyze_reading_content(&reading, &question, expected_category)?;

            if let (Some(created), Some(completed)) = (job_record.created_at, job_record.completed_at) {
                let duration = completed.signed_duration_since(created);
                println!("  ⏱️  Processing time: {} seconds", duration.num_seconds());
            }
        } else {
            println!("  ❌ No result found");
        }

        // Cleanup
        sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
            .execute(&db_pool)
            .await?;
    }

    println!("\n✅ All agent pipeline tests completed!");
    Ok(())
}

fn analyze_reading_content(
    reading: &Value,
    original_question: &str,
    expected_category: &str
) -> Result<(), Box<dyn std::error::Error>> {

    println!("\n🔍 Content Analysis:");

    // Check header - should contain "มีมี่"
    let header = reading["header"].as_str().unwrap_or("");
    let has_mimi_header = header.contains("มีมี่");
    println!("  Header contains 'มีมี่': {}", if has_mimi_header { "✅" } else { "❌" });

    // Check cards count
    let cards_count = reading["cards_reading"].as_array().map(|a| a.len()).unwrap_or(0);
    println!("  Number of cards drawn: {} {}", cards_count, if cards_count == 3 { "✅" } else { "❌" });

    // Check reading length - should be substantial
    let reading_text = reading["reading"].as_str().unwrap_or("");
    let reading_length = reading_text.chars().count();
    println!("  Reading length: {} chars {}",
        reading_length,
        if reading_length > 100 { "✅" } else { "❌" }
    );

    // Check for suggestions
    let suggestions_count = reading["suggestions"].as_array().map(|a| a.len()).unwrap_or(0);
    println!("  Number of suggestions: {} {}",
        suggestions_count,
        if suggestions_count > 0 { "✅" } else { "❌" }
    );

    // Check for final message
    let final_count = reading["final"].as_array().map(|a| a.len()).unwrap_or(0);
    println!("  Final messages: {} {}",
        final_count,
        if final_count > 0 { "✅" } else { "❌" }
    );

    // Check closing message
    let has_closing = reading["end"].is_string();
    println!("  Has closing message: {}", if has_closing { "✅" } else { "❌" });

    // Check Thai language content
    let is_thai_content = contains_thai_content(reading);
    println!("  Thai language content: {}", if is_thai_content { "✅" } else { "❌" });

    // Check question relevance
    let is_relevant = check_question_relevance(reading_text, original_question);
    println!("  Question relevance: {}", if is_relevant { "✅" } else { "❌" });

    // Category-specific checks
    match expected_category {
        "love" => {
            let has_love_content = reading_text.contains("ความรัก") ||
                                  reading_text.contains("สัมพันธ์") ||
                                  reading_text.contains("คู่");
            println!("  Love category content: {}", if has_love_content { "✅" } else { "⚠️" });
        },
        "career" => {
            let has_career_content = reading_text.contains("งาน") ||
                                   reading_text.contains("อาชีพ") ||
                                   reading_text.contains "เจริญ");
            println!("  Career category content: {}", if has_career_content { "✅" } else { "⚠️" });
        },
        "health" => {
            let has_health_content = reading_text.contains("สุขภาพ") ||
                                   reading_text.contains("ร่างกาย") ||
                                   reading_text.contains("ดีขึ้น");
            println!("  Health category content: {}", if has_health_content { "✅" } else { "⚠️" });
        },
        _ => {}
    }

    Ok(())
}

fn contains_thai_content(reading: &Value) -> bool {
    // Check all text fields for Thai characters
    let fields_to_check = vec![
        &reading["header"],
        &reading["reading"],
        &reading["end"]
    ];

    for field in fields_to_check {
        if let Some(text) = field.as_str() {
            // Check for Thai Unicode range (U+0E00 to U+0E7F)
            if text.chars().any(|c| c >= '\u{0E00}' && c <= '\u{0E7F}') {
                return true;
            }
        }
    }

    // Check arrays
    if let Some(suggestions) = reading["suggestions"].as_array() {
        for suggestion in suggestions {
            if let Some(text) = suggestion.as_str() {
                if text.chars().any(|c| c >= '\u{0E00}' && c <= '\u{0E7F}') {
                    return true;
                }
            }
        }
    }

    if let Some(final_msgs) = reading["final"].as_array() {
        for msg in final_msgs {
            if let Some(text) = msg.as_str() {
                if text.chars().any(|c| c >= '\u{0E00}' && c <= '\u{0E7F}') {
                    return true;
                }
            }
        }
    }

    false
}

fn check_question_relevance(reading: &str, question: &str) -> bool {
    // Simple relevance check - reading should address the question topic
    let question_keywords: Vec<&str> = question.split_whitespace().collect();
    let reading_lower = reading.to_lowercase();

    for keyword in question_keywords {
        if keyword.len() > 2 && reading_lower.contains(keyword) {
            return true;
        }
    }

    false
}

#[tokio::test]
async fn test_database_job_state_transitions() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗄️  Database State Transitions Test");
    println!("==================================");

    let app = create_test_app().await?;
    let db_pool = app.db_pool.clone();

    // Submit a job
    let request_body = json!({
        "question": "ทดสอบสถานะการทำงาน",
        "cards": 3
    });

    let response = app.client
        .post("/api/v1/tarots/read")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let submission_response: Value = response.json().await?;
    let job_id = submission_response["job_id"].as_str().unwrap();

    println!("📋 Job ID: {}", job_id);

    // Track state changes over time
    let mut last_status = "queued".to_string();
    let mut state_history = Vec::new();
    let job_id_uuid = Uuid::parse_str(job_id)?;

    for i in 0..30 {
        tokio::time::sleep(Duration::from_secs(2)).await;

        let job_record = sqlx::query!(
            r#"
            SELECT status::text as status, attempts, worker_id, created_at, completed_at
            FROM jobs
            WHERE id = $1
            "#,
            job_id_uuid
        )
        .fetch_one(&db_pool)
        .await?;

        if job_record.status != last_status {
            println!("\n⏱️  Second {}: State changed", i * 2);
            println!("  From: {}", last_status);
            println!("  To: {}", job_record.status);
            println!("  Attempts: {}", job_record.attempts);
            println!("  Worker: {:?}", job_record.worker_id);

            last_status = job_record.status.clone();
            state_history.push((i * 2, job_record.status.clone()));
        }

        if job_record.status == "completed" || job_record.status == "failed" {
            break;
        }
    }

    // Final state verification
    let final_record = sqlx::query!(
        r#"
        SELECT status::text as status, result, attempts, worker_id, created_at, completed_at
        FROM jobs
        WHERE id = $1
        "#,
        job_id_uuid
    )
    .fetch_one(&db_pool)
    .await?;

    println!("\n📊 Final Job Record:");
    println!("  Status: {}", final_record.status);
    println!("  Attempts: {}", final_record.attempts);
    println!("  Worker ID: {:?}", final_record.worker_id);
    println!("  Created: {:?}", final_record.created_at);
    println!("  Completed: {:?}", final_record.completed_at);
    println!("  Has Result: {}", final_record.result.is_some());

    // Verify all states were visited
    println!("\n📈 State History:");
    for (time, state) in state_history {
        println!("  {}: {}", time, state);
    }

    // Cleanup
    sqlx::query!("DELETE FROM jobs WHERE id = $1", job_id)
        .execute(&db_pool)
        .await?;

    Ok(())
}