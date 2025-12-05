//! Complete End-to-End Tarot Reading Flow Test
//!
//! This test demonstrates the complete workflow:
//! 1. User submits question via API
//! 2. System queues the job
//! 3. Worker processes through 3 agents
//! 4. Result stored in database
//! 5. User retrieves completed reading

use redis::Client;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

mod common;
use common::setup::create_test_app;

#[tokio::test]
async fn complete_tarot_reading_flow_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test environment
    println!("\n🔮 Testing Complete Tarot Reading Flow");
    println!("=====================================\n");

    let app = create_test_app().await?;
    let db_pool = app.db_pool.clone();
    let redis_client = app.redis_client.clone();

    // Step 1: Clean up any existing data
    println!("🧹 Setting up test environment...");
    cleanup_test_data(&db_pool).await?;

    // Step 2: Submit question via API
    println!("\n📝 Step 1: Submitting Question");
    println!("---------------------------");

    let test_question = "ชีวิตฉันจะดีขึ้นไหมในเดือนหน้า";
    let request_body = json!({
        "question": test_question,
        "cards": 3
    });

    println!("Question: {}", test_question);
    println!("Cards: {}", request_body["cards"]);

    let response = app
        .client
        .post("/api/v1/tarots/read")
        .json(&request_body)
        .send()
        .await?;

    assert_eq!(response.status(), 200);

    let submission_response: Value = response.json().await?;
    let job_id = submission_response["job_id"].as_str().unwrap();

    println!("✅ Job submitted successfully");
    println!("📋 Job ID: {}", job_id);

    // Step 3: Verify queue and database state
    println!("\n🗄️  Step 2: Verifying Job Creation");
    println!("---------------------------------");

    // Check database
    let job_id_uuid = Uuid::parse_str(job_id)?;
    let job_record = sqlx::query!(
        r#"
        SELECT id, status::text as status, payload, created_at
        FROM jobs
        WHERE id = $1
        "#,
        job_id_uuid
    )
    .fetch_one(&db_pool)
    .await?;

    println!("✅ Job found in database");
    println!("📊 Status: {}", job_record.status);
    println!("📅 Created at: {}", job_record.created_at);

    // Check Redis queue
    let queue_length = get_redis_queue_length(&redis_client).await?;
    println!("📦 Queue length: {}", queue_length);

    assert_eq!(job_record.status, "queued");

    // Step 4: Wait for worker to process
    println!("\n⏳ Step 3: Waiting for Processing");
    println!("------------------------------");

    let mut poll_count = 0;
    let max_polls = 120; // 2 minutes max

    loop {
        let status_response = app
            .client
            .get(&format!("/api/v1/tarots/{}", job_id))
            .send()
            .await?;

        assert_eq!(status_response.status(), 200);

        let status_result: Value = status_response.json().await?;
        let status = status_result["status"].as_str().unwrap();

        print!("🔄 Poll {}: Status = {} \r", poll_count + 1, status);
        std::io::Write::flush(&mut std::io::stdout())?;

        match status {
            "completed" => {
                println!("\n✅ Processing completed!");
                break;
            }
            "failed" => {
                println!("\n❌ Processing failed!");
                println!("Error: {:?}", status_result["error"]);
                return Err("Job processing failed".into());
            }
            _ => {
                poll_count += 1;
                if poll_count >= max_polls {
                    return Err("Processing timeout".into());
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }

    // Step 5: Retrieve and display complete result
    println!("\n📖 Step 4: Retrieving Final Reading");
    println!("--------------------------------");

    let final_response = app
        .client
        .get(&format!("/api/v1/tarots/{}", job_id))
        .send()
        .await?;

    let final_result: Value = final_response.json().await?;

    println!("✅ Final result retrieved");
    println!("🎯 Job Status: {}", final_result["status"]);
    println!("📅 Completed at: {:?}", final_result["completed_at"]);

    // Step 6: Verify final database state
    println!("\n🗄️  Step 5: Verifying Final Database State");
    println!("------------------------------------------");

    let final_job_record = sqlx::query!(
        r#"
        SELECT id, status::text as status, result, completed_at, attempts, worker_id
        FROM jobs
        WHERE id = $1
        "#,
        job_id_uuid
    )
    .fetch_one(&db_pool)
    .await?;

    println!("✅ Final job record verified");
    println!("📊 Final Status: {}", final_job_record.status);
    println!("🔢 Attempts: {}", final_job_record.attempts);
    println!("🤖 Worker ID: {:?}", final_job_record.worker_id);
    println!("📅 Completed: {:?}", final_job_record.completed_at);

    // Step 7: Display the actual reading result
    println!("\n🔮 TAROT READING RESULT");
    println!("=======================");

    if let Some(result) = final_job_record.result {
        let reading: Value = serde_json::from_value(result)?;

        println!("\n📜 Header:");
        println!("{}", reading["header"]);

        println!("\n🃏 Cards Drawn:");
        if let Some(cards) = reading["cards_reading"].as_array() {
            for (i, card) in cards.iter().enumerate() {
                println!("  Card {}: {}", i + 1, card);
            }
        }

        println!("\n📖 Reading Interpretation:");
        println!("{}", reading["reading"]);

        if let Some(suggestions) = reading["suggestions"].as_array() {
            println!("\n💡 Suggestions:");
            for suggestion in suggestions {
                println!("  • {}", suggestion);
            }
        }

        if let Some(final_msg) = reading["final"].as_array() {
            println!("\n🎯 Final Words:");
            for msg in final_msg {
                println!("  {}", msg);
            }
        }

        println!("\n🙏 Closing:");
        println!("{}", reading["end"]);
    }

    // Step 8: Verification summary
    println!("\n✅ VERIFICATION SUMMARY");
    println!("======================");

    // Verify all required fields exist
    let result = final_job_record.result.unwrap();
    let result_json: Value = serde_json::from_value(result)?;

    let mut checks = vec![];

    checks.push(("Header exists", result_json["header"].is_string()));
    checks.push((
        "Cards reading exists",
        result_json["cards_reading"].is_array(),
    ));
    checks.push((
        "Reading interpretation exists",
        result_json["reading"].is_string(),
    ));
    checks.push(("Suggestions exist", result_json["suggestions"].is_array()));
    checks.push(("Final message exists", result_json["final"].is_array()));
    checks.push(("End message exists", result_json["end"].is_string()));

    let all_passed = checks.iter().all(|(_, passed)| *passed);

    for (check_name, passed) in checks {
        let icon = if passed { "✅" } else { "❌" };
        println!("{} {}", icon, check_name);
    }

    if all_passed {
        println!("\n🎉 ALL CHECKS PASSED! Complete tarot reading flow is working correctly!");
    } else {
        println!("\n⚠️  Some checks failed. Please review the output above.");
    }

    // Performance metrics
    if let (Some(created), Some(completed)) =
        (final_job_record.created_at, final_job_record.completed_at)
    {
        let duration = completed.signed_duration_since(created);
        println!("\n⏱️  Processing time: {} seconds", duration.num_seconds());
    }

    // Cleanup
    cleanup_test_data(&db_pool).await?;

    Ok(())
}

async fn cleanup_test_data(db_pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM jobs WHERE payload->>'question' LIKE 'ชีวิตฉันจะดีขึ้นไหม%'")
        .execute(db_pool)
        .await?;
    Ok(())
}

async fn get_redis_queue_length(redis_client: &Client) -> Result<i64, redis::RedisError> {
    let mut conn = redis_client.get_async_connection().await?;
    let length: i64 = redis::cmd("LLEN")
        .arg("tarot_jobs")
        .query_async(&mut conn)
        .await?;
    Ok(length)
}
