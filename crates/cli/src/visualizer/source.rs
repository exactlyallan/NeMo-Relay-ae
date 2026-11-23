// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Read adapters only. Collection, authorization and retention remain owned by Relay.

use nemo_relay_types::diagnostics::{
    DiagnosticHttpError, DiagnosticHttpReason, DiagnosticNumber, DiagnosticSnapshot,
};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct SnapshotSource {
    pub url: String,
    pub daemon: bool,
    pub worker: Option<Uuid>,
    pub max_bytes: usize,
    pub demo: bool,
}

pub(super) struct FrameRead {
    pub diagnostics: Result<SnapshotRead, ReadError>,
    pub status: Result<super::status::RuntimeStatus, &'static str>,
    pub sampled_at: std::time::Instant,
}

pub(super) struct SnapshotRead {
    pub snapshot: DiagnosticSnapshot,
    pub cache: Option<CacheStatus>,
}

#[derive(Deserialize)]
pub(super) struct CacheStatus {
    pub availability: String,
    pub latest_poll: String,
    pub age_millis: Option<DiagnosticNumber>,
}

pub(super) struct ReadError {
    pub message: String,
    /// Authorization loss, a missing assignment or invalid schema invalidates old evidence.
    pub discard: bool,
    pub retryable: bool,
}

impl ReadError {
    fn invalid(message: &str) -> Self {
        Self {
            message: message.into(),
            discard: true,
            retryable: false,
        }
    }
}

impl SnapshotSource {
    pub(super) fn collect(&self, frame: u64) -> FrameRead {
        let status = if self.demo {
            Ok(super::demo::status(frame))
        } else {
            super::status::read(&self.url, self.daemon, self.worker)
        };
        let sampled_at = std::time::Instant::now();
        // Independent reads: lack of diagnostic authority must not erase public status.
        FrameRead {
            diagnostics: self.read(frame),
            status,
            sampled_at,
        }
    }

    pub(super) fn read(&self, demo_frame: u64) -> Result<SnapshotRead, ReadError> {
        if self.demo {
            return Ok(SnapshotRead {
                snapshot: super::demo::snapshot(demo_frame),
                cache: None,
            });
        }
        let path = self.worker.map_or_else(
            || crate::diagnostics_http::PREFIX.to_owned(),
            |worker| format!("/_nemo_relay/v1/workers/{worker}/diagnostics"),
        );
        let response =
            crate::gateway::client::get_diagnostics(&self.url, &path, self.max_bytes, self.daemon)
                .map_err(|_| {
                    ReadError {
            // Transport errors may contain paths/addresses. Do not project them into the board.
            message:
                "Connection unavailable; check listener, pinned identity and operator credentials"
                    .into(),
            discard: true,
            retryable: true,
        }
                })?;
        decode_response(response.status, &response.body, self.worker.is_some())
    }

    pub(super) fn label(&self) -> String {
        if self.demo {
            "DEMO / illustrative values / no listener connected".into()
        } else if let Some(worker) = self.worker {
            format!("SELECTED WORKER / {worker}")
        } else if self.daemon {
            "DAEMON / all reported workers".into()
        } else {
            "GATEWAY / selected local instance".into()
        }
    }
}

/// Worker routes wrap a canonical snapshot. Preserve cache freshness separately from read success.
fn decode_response(status: u16, body: &[u8], worker: bool) -> Result<SnapshotRead, ReadError> {
    if status != 200 {
        let reason = DiagnosticHttpError::from_response(status, body).map(|e| e.error.reason);
        let message = match reason {
            Some(DiagnosticHttpReason::Missing) if worker => "Worker not found".into(),
            Some(reason) => serde_json::to_string(&reason)
                .unwrap_or_else(|_| "unknown_reason".into())
                .trim_matches('"')
                .replace('_', " "),
            None => format!("Diagnostics endpoint unavailable (HTTP {status}; unknown reason)"),
        };
        let retryable = matches!(
            reason,
            Some(DiagnosticHttpReason::Busy | DiagnosticHttpReason::ShuttingDown)
        );
        return Err(ReadError {
            message,
            discard: !retryable,
            retryable,
        });
    }
    if worker {
        #[derive(Deserialize)]
        struct Assignment {
            source: Uuid,
        }
        #[derive(Deserialize)]
        struct WorkerPage {
            assignment: Assignment,
            collection: CacheStatus,
            snapshot: Option<DiagnosticSnapshot>,
        }
        let page: WorkerPage = serde_json::from_slice(body)
            .map_err(|_| ReadError::invalid("Unsupported or malformed worker snapshot"))?;
        let snapshot = page.snapshot.ok_or_else(|| ReadError {
            message: "Worker cache has no snapshot yet".into(),
            discard: true,
            retryable: true,
        })?;
        if snapshot.source.instance_id != page.assignment.source {
            return Err(ReadError::invalid(
                "Worker assignment and snapshot source differ",
            ));
        }
        Ok(SnapshotRead {
            snapshot,
            cache: Some(page.collection),
        })
    } else {
        let snapshot = serde_json::from_slice(body)
            .map_err(|_| ReadError::invalid("Unsupported or malformed diagnostic snapshot"))?;
        Ok(SnapshotRead {
            snapshot,
            cache: None,
        })
    }
}

#[cfg(test)]
#[path = "../../tests/coverage/visualizer/source_tests.rs"]
mod tests;

/// Local wall time is only a correlation aid, never a cross-source elapsed-time clock.
pub(super) fn now() -> nemo_relay_types::diagnostics::ObservationTime {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    nemo_relay_types::diagnostics::ObservationTime::from_timestamp_millis(
        millis.min(i64::MAX as u128) as i64,
    )
    .unwrap()
}
