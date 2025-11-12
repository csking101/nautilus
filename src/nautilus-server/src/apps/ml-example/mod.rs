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
    // Debug: print contents of current working directory
    match std::fs::read_dir(".") {
        Ok(entries) => {
            println!("Current directory contents:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("{}", entry.path().display());
                }
            }
        }
        Err(e) => {
            println!("Failed to read current directory: {}", e);
        }
    }
    // Debug: check if /ml_task.bin is executable
    match std::fs::metadata("/ml_task.bin") {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = permissions.mode();
                let is_executable = mode & 0o111 != 0;
                println!(
                    "/ml_task.bin exists. Permissions: {:o}. Executable: {}",
                    mode, is_executable
                );
            }
            #[cfg(not(unix))]
            {
                println!("/ml_task.bin exists. Permissions: {:?}", permissions);
            }
        }
        Err(e) => {
            println!("/ml_task.bin not found: {}", e);
        }
    }
    // Run shell command from payload
    let output = Command::new("sh")
        .arg("-c")
        .arg(&request.payload.data_path)
        .output();

    let output = match output {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "Failed to run shell command. Status: {:?}\nStdout: {}\nStderr: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err(EnclaveError::GenericError(
                    format!(
                        "Shell command error: status={:?}, stdout={}, stderr={}",
                        output.status,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    ),
                ));
            }
            output
        }
        Err(e) => {
            println!("Failed to execute shell command: {}", e);
            return Err(EnclaveError::GenericError(format!(
                "Failed to run shell command: {}",
                e
            )));
        }
    };

    let result = String::from_utf8(output.stdout)
        .map_err(|e| EnclaveError::GenericError(format!("Invalid Python output: {}", e)))?;

    println!("Raw shell output: {}", result);

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
