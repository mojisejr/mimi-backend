//! Tarot Reading API Routes
//!
//! Endpoints for submitting tarot reading requests and retrieving results.

use crate::{
    models::{ErrorResponse, HealthResponse, TarotRequest, TarotResponse},
    queue::TarotQueue,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// API state containing shared resources
#[derive(Clone)]
pub struct ApiState {
    pub redis_client: redis::Client,
    pub tarot_queue: Arc<TarotQueue>,
}

/// Request a tarot reading
///
/// POST /api/v1/tarots/read
///
/// Request body:
/// ```json
/// {
///   "question": "ควรจะลงทุนอะไรดีครับ",
///   "user_id": "optional-user-id",
/// }
/// ```
///
/// Response:
/// ```json
/// {
///   "job_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "queued",
///   "message": "Tarot reading request submitted successfully",
///   "created_at": "2023-12-02T10:30:00Z",
///   "estimated_wait_seconds": 60
/// }
/// ```
pub async fn request_reading(
    State(state): State<ApiState>,
    _headers: HeaderMap,
    Json(request): Json<TarotRequest>,
) -> Result<Json<TarotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Step 1: Validate the request
    if let Err(validation_error) = request.validate() {
        let error_response = ErrorResponse::new(validation_error.to_string());
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Step 2: Submit reading request using TarotQueue
    let question = request.get_trimmed_question();
    let user_id = request.user_id.as_deref();
    let card_count = Some(3); // Default to 3 cards

    match state
        .tarot_queue
        .submit_reading_request(&question, user_id, card_count)
        .await
    {
        Ok(submission_result) => {
            // Step 3: Return successful response
            let response = TarotResponse::queued(submission_result.job_id);
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Failed to submit reading request: {}", e);
            let error_response = ErrorResponse::with_code(
                "Failed to submit tarot reading request".to_string(),
                "QUEUE_ERROR".to_string(),
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get a previous tarot reading result
///
/// GET /api/v1/tarots/{job_id}
///
/// Response:
/// ```json
/// {
///   "job_id": "550e8400-e29b-41d4-a716-446655440000",
///   "status": "completed",
///   "result": { ... },
///   "created_at": "2023-12-02T10:30:00Z",
///   "completed_at": "2023-12-02T10:31:00Z"
/// }
/// ```
pub async fn get_reading(
    State(state): State<ApiState>,
    axum::extract::Path(job_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    // Get reading result using TarotQueue
    match state.tarot_queue.get_reading_result(job_id).await {
        Ok(Some(reading_result)) => {
            let response_json = json!({
                "job_id": reading_result.job_id,
                "status": reading_result.status,
                "question": reading_result.question,
                "result": reading_result.result,
                "created_at": reading_result.created_at,
                "started_at": reading_result.started_at,
                "completed_at": reading_result.completed_at,
                "error_message": reading_result.error_message
            });
            Ok(Json(response_json))
        }
        Ok(None) => {
            let error_response = ErrorResponse::with_code(
                "Job not found or not completed".to_string(),
                "JOB_NOT_FOUND".to_string(),
            );
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            eprintln!("Failed to get reading result: {}", e);
            let error_response = ErrorResponse::with_code(
                "Failed to retrieve reading result".to_string(),
                "RETRIEVAL_ERROR".to_string(),
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Health check endpoint
///
/// GET /api/v1/health
///
/// Response:
/// ```json
/// {
///   "status": "healthy",
///   "service": "mimivibe-backend-api",
///   "timestamp": "2023-12-02T10:30:00Z",
///   "version": "0.1.0"
/// }
/// ```
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse::healthy())
}
