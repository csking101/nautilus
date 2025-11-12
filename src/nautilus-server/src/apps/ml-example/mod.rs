// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::common::IntentMessage;
use crate::common::{to_signed_response, IntentScope, ProcessDataRequest, ProcessedDataResponse};
use crate::AppState;
use crate::EnclaveError;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::process::Command;

/// ====
/// ML Example: Calls Python script for ML task
/// ====

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MLResponse {
    pub accuracy: u64,
    pub loss: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MLRequest {
    pub data_path: String,
}

pub async fn process_data(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProcessDataRequest<MLRequest>>,
) -> Result<Json<ProcessedDataResponse<IntentMessage<MLResponse>>>, EnclaveError> {
    // Call compiled Python binary
    let output = Command::new("ml_task")
        .arg(&request.payload.data_path)
        .output()
        .map_err(|e| EnclaveError::GenericError(format!("Failed to run Python binary: {}", e)))?;

    if !output.status.success() {
        return Err(EnclaveError::GenericError(
            format!("Python script error: {:?}", output.stderr),
        ));
    }

    let result = String::from_utf8(output.stdout)
        .map_err(|e| EnclaveError::GenericError(format!("Invalid Python output: {}", e)))?;

    let ml_result: MLResponse = serde_json::from_str(&result)
        .map_err(|e| EnclaveError::GenericError(format!("Failed to parse ML result: {}", e)))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| EnclaveError::GenericError(format!("Failed to get timestamp: {}", e)))?
        .as_millis() as u64;

    Ok(Json(to_signed_response(
        &state.eph_kp,
        ml_result,
        timestamp,
        IntentScope::ProcessData,
    )))
}
