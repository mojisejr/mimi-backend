//! Complete End-to-End System Validation Tests
//!
//! This test file validates the entire tarot reading system works end-to-end
//! after implementing all previous slices. The validation tests the complete
//! 5-step flow:
//! 1. User submits question via API
//! 2. Job is queued in database
//! 3. Worker polls and processes job
//! 4. 3-agent pipeline generates reading
//! 5. User retrieves completed reading

use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

mod setup; // AUTO-LOAD .env via tests/setup.rs

#[tokio::test]
async fn test_complete_tarot_reading_flow() {
    setup::setup();

    let client = Client::new();

    // 1. Submit reading request
    let request_body = json!({
        "question": "ความรักของฉันจะเป็นอย่างไร",
        "cards": 3
    });

    println!("🔮 Submitting tarot reading request...");
    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .header("Authorization", "Bearer default-key")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to submit request");

    assert_eq!(response.status(), 200, "API should accept request");

    let submission: Value = response.json().await.unwrap();
    let job_id = submission["job_id"].as_str().unwrap();
    assert_eq!(
        submission["status"], "queued",
        "Initial status should be queued"
    );

    println!("✅ Request submitted with job_id: {}", job_id);

    // 2. Wait for processing (max 60 seconds)
    let mut final_status = "queued".to_string();
    println!("⏳ Monitoring job status...");

    for attempt in 0..30 {
        sleep(Duration::from_secs(2)).await;

        let status_response = client
            .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
            .header("Authorization", "Bearer default-key")
            .send()
            .await
            .expect("Failed to check status");

        assert_eq!(status_response.status(), 200, "Status check should succeed");

        let status_result: Value = status_response.json().await.unwrap();
        final_status = status_result["status"].as_str().unwrap().to_string();

        println!("  Attempt {}: status = {}", attempt + 1, final_status);

        if final_status == "completed" || final_status == "failed" {
            break;
        }
    }

    // 3. Verify completion
    assert_eq!(
        final_status, "completed",
        "Job should complete successfully"
    );

    // 4. Retrieve and validate reading
    println!("📖 Retrieving completed reading...");
    let final_response = client
        .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
        .header("Authorization", "Bearer default-key")
        .send()
        .await
        .expect("Failed to get final result");

    let final_result: Value = final_response.json().await.unwrap();

    // Validate reading structure
    assert!(
        final_result["result"].is_object(),
        "Result should be an object"
    );
    let result = &final_result["result"];

    assert!(result["header"].is_string(), "Header should be a string");
    assert!(
        result["cards_reading"].is_array(),
        "Cards reading should be an array"
    );
    assert!(result["reading"].is_string(), "Reading should be a string");
    assert!(
        result["suggestions"].is_array(),
        "Suggestions should be an array"
    );
    assert!(result["final"].is_array(), "Final should be an array");
    assert!(result["end"].is_string(), "End should be a string");

    // Verify Thai content
    let reading = result["reading"].as_str().unwrap();
    assert!(!reading.is_empty(), "Reading should not be empty");
    // Should contain Thai characters
    assert!(
        reading.chars().any(|c| c >= 'ก' && c <= 'ฮ'),
        "Reading should contain Thai characters"
    );

    println!("✅ Complete tarot reading flow validated!");
    println!("📖 Reading: {}", reading);
}

#[tokio::test]
async fn test_five_card_reading_flow() {
    setup::setup();

    let client = Client::new();

    // Submit 5-card reading request
    let request_body = json!({
        "question": "อนาคตการงานของฉันจะเป็นอย่างไร",
        "cards": 5
    });

    println!("🔮 Submitting 5-card tarot reading request...");
    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .header("Authorization", "Bearer default-key")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to submit request");

    assert_eq!(response.status(), 200);

    let submission: Value = response.json().await.unwrap();
    let job_id = submission["job_id"].as_str().unwrap();
    assert_eq!(submission["status"], "queued");

    println!("✅ 5-card request submitted with job_id: {}", job_id);

    // Wait for completion
    let mut final_status = "queued".to_string();
    for _attempt in 0..30 {
        sleep(Duration::from_secs(2)).await;

        let status_response = client
            .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
            .header("Authorization", "Bearer default-key")
            .send()
            .await
            .expect("Failed to check status");

        let status_result: Value = status_response.json().await.unwrap();
        final_status = status_result["status"].as_str().unwrap().to_string();

        if final_status == "completed" || final_status == "failed" {
            break;
        }
    }

    assert_eq!(
        final_status, "completed",
        "5-card reading should complete successfully"
    );

    // Verify 5 cards were selected
    let final_response = client
        .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
        .header("Authorization", "Bearer default-key")
        .send()
        .await
        .expect("Failed to get final result");

    let final_result: Value = final_response.json().await.unwrap();
    let cards = &final_result["result"]["cards_reading"].as_array().unwrap();
    assert_eq!(cards.len(), 5, "Should have exactly 5 cards");

    println!(
        "✅ 5-card reading flow validated with {} cards",
        cards.len()
    );
}

#[tokio::test]
async fn test_multiple_job_processing() {
    setup::setup();

    let client = Client::new();
    let mut job_ids = Vec::new();

    // Submit multiple requests simultaneously
    println!("🔮 Submitting multiple reading requests...");
    for i in 0..3 {
        let request_body = json!({
            "question": format!("คำถามที่ {} เกี่ยวกับชีวิตของฉัน", i + 1),
            "cards": 3
        });

        let response = client
            .post("http://localhost:3000/api/v1/tarots/read")
            .header("Authorization", "Bearer default-key")
            .json(&request_body)
            .send()
            .await
            .expect("Failed to submit request");

        assert_eq!(response.status(), 200);

        let submission: Value = response.json().await.unwrap();
        let job_id = submission["job_id"].as_str().unwrap().to_string();
        job_ids.push(job_id);

        println!("  Submitted request {}: {}", i + 1, job_ids[i]);
    }

    // Wait for all jobs to complete
    println!("⏳ Waiting for all jobs to complete...");
    let mut completed_count = 0;
    let mut job_completed = vec![false; job_ids.len()];

    for _attempt in 0..60 {
        // Longer timeout for multiple jobs
        sleep(Duration::from_secs(2)).await;

        for (index, job_id) in job_ids.iter().enumerate() {
            if job_completed[index] {
                continue;
            }

            let status_response = client
                .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
                .header("Authorization", "Bearer default-key")
                .send()
                .await
                .expect("Failed to check status");

            if status_response.status() == 200 {
                let status_result: Value = status_response.json().await.unwrap();
                let status = status_result["status"].as_str().unwrap();

                if status == "completed" {
                    job_completed[index] = true;
                    completed_count += 1;
                    println!("  Job {} completed", job_id);
                }
            }
        }

        if completed_count == 3 {
            break;
        }
    }

    assert_eq!(
        completed_count, 3,
        "All 3 jobs should complete successfully"
    );
    println!("✅ Multiple job processing validated");
}

#[tokio::test]
async fn test_error_handling_flow() {
    setup::setup();

    let client = Client::new();

    // Test 1: Invalid card count
    println!("❌ Testing invalid card count...");
    let request_body = json!({
        "question": "คำถามปกติ",
        "cards": 7  // Invalid card count
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .header("Authorization", "Bearer default-key")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to submit request");

    // API should accept it but worker should fail
    assert_eq!(
        response.status(),
        200,
        "API should accept request initially"
    );

    let submission: Value = response.json().await.unwrap();
    let job_id = submission["job_id"].as_str().unwrap();

    // Check that job eventually fails
    let mut final_status = "queued".to_string();
    for _ in 0..30 {
        sleep(Duration::from_secs(2)).await;

        let status_response = client
            .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
            .header("Authorization", "Bearer default-key")
            .send()
            .await
            .expect("Failed to check status");

        let status_result: Value = status_response.json().await.unwrap();
        final_status = status_result["status"].as_str().unwrap().to_string();

        if final_status == "completed" || final_status == "failed" {
            break;
        }
    }

    assert_eq!(
        final_status, "failed",
        "Job with invalid card count should fail"
    );

    // Test 2: Empty question
    println!("❌ Testing empty question...");
    let request_body = json!({
        "question": "",
        "cards": 3
    });

    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .header("Authorization", "Bearer default-key")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to submit request");

    assert_eq!(
        response.status(),
        400,
        "Empty question should be rejected immediately"
    );

    let error: Value = response.json().await.unwrap();
    assert!(error["error"].is_string(), "Should return error message");

    println!("✅ Error handling flow validated");
}

#[tokio::test]
async fn test_processing_time() {
    setup::setup();

    let client = Client::new();

    let request_body = json!({
        "question": "คำถามสำหรับทดสอบประสิทธิภาพ",
        "cards": 3
    });

    println!("⏱️  Testing processing time...");
    let start_time = std::time::Instant::now();

    // Submit request
    let response = client
        .post("http://localhost:3000/api/v1/tarots/read")
        .header("Authorization", "Bearer default-key")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to submit request");

    assert_eq!(response.status(), 200);

    let submission: Value = response.json().await.unwrap();
    let job_id = submission["job_id"].as_str().unwrap();

    // Wait for completion
    let mut final_status = "queued".to_string();
    for _ in 0..30 {
        sleep(Duration::from_secs(2)).await;

        let status_response = client
            .get(&format!("http://localhost:3000/api/v1/tarots/{}", job_id))
            .header("Authorization", "Bearer default-key")
            .send()
            .await
            .expect("Failed to check status");

        let status_result: Value = status_response.json().await.unwrap();
        final_status = status_result["status"].as_str().unwrap().to_string();

        if final_status == "completed" || final_status == "failed" {
            break;
        }
    }

    let total_time = start_time.elapsed();

    assert_eq!(final_status, "completed", "Job should complete");
    assert!(
        total_time.as_secs() < 60,
        "Processing should complete within 60 seconds"
    );

    println!(
        "✅ Processing time validated: {} seconds",
        total_time.as_secs()
    );
}
