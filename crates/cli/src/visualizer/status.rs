// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Independent live status adapter. Status is not an authorization grant or an activity log.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WorkerStatus {
    pub worker_id: Uuid,
    pub state: String,
    pub reference_count: u64,
    pub control_available: bool,
    pub in_flight: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct Health {
    pub service: String,
    pub status: String,
    pub version: String,
    pub instance_id: String,
    #[serde(default)]
    pub deployment_mode: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct RuntimeStatus {
    pub health: Health,
    /// None means unavailable, never a fabricated empty worker list.
    pub workers: Option<Vec<WorkerStatus>>,
    pub worker_error: Option<&'static str>,
    pub selected_worker: Option<Uuid>,
}

impl RuntimeStatus {
    pub fn requests(&self) -> Option<u64> {
        self.workers
            .as_ref()?
            .iter()
            .try_fold(0_u64, |sum, w| sum.checked_add(w.in_flight))
    }
    pub fn sessions(&self) -> Option<u64> {
        self.workers
            .as_ref()?
            .iter()
            .try_fold(0_u64, |sum, w| sum.checked_add(w.reference_count))
    }
}

pub(super) fn read(
    url: &str,
    daemon: bool,
    worker: Option<Uuid>,
) -> Result<RuntimeStatus, &'static str> {
    let health = crate::gateway::client::get_visualizer_status(url, "/healthz")
        .map_err(|_| "Cannot reach the selected Relay instance")?;
    let health = decode_health(health.status, &health.body, daemon)?;
    if !daemon {
        return Ok(RuntimeStatus {
            health,
            workers: None,
            worker_error: None,
            selected_worker: None,
        });
    }
    let path = worker.map_or_else(
        || "/_nemo_relay/v1/workers/status".into(),
        |id| format!("/_nemo_relay/v1/workers/{id}/status"),
    );
    let response = crate::gateway::client::get_visualizer_status(url, &path);
    let workers = response
        .map_err(|_| "Worker status could not be read")
        .and_then(|r| decode_workers(r.status, &r.body, &health.instance_id, worker));
    let (workers, worker_error) = match workers {
        Ok(workers) => (Some(workers), None),
        Err(reason) => (None, Some(reason)),
    };
    Ok(RuntimeStatus {
        health,
        workers,
        worker_error,
        selected_worker: worker,
    })
}

fn decode_health(status: u16, body: &[u8], daemon: bool) -> Result<Health, &'static str> {
    if status != 200 {
        return Err("Relay health is unavailable");
    }
    let health: Health =
        serde_json::from_slice(body).map_err(|_| "Unrecognized Relay health response")?;
    let expected = if daemon {
        "nemo-relay-daemon"
    } else {
        "nemo-relay"
    };
    if health.service != expected || health.status != "ok" || health.instance_id.is_empty() {
        return Err("The selected address did not report the expected Relay service");
    }
    Ok(health)
}

fn decode_workers(
    status: u16,
    body: &[u8],
    instance: &str,
    worker: Option<Uuid>,
) -> Result<Vec<WorkerStatus>, &'static str> {
    if status == 404 && worker.is_some() {
        return Err("The selected worker has ended or been replaced");
    }
    if status != 200 {
        return Err("Worker status is unavailable");
    }
    #[derive(Deserialize)]
    struct Envelope {
        instance_id: String,
        #[serde(default)]
        worker: Option<WorkerStatus>,
        #[serde(default)]
        workers: Option<Vec<WorkerStatus>>,
    }
    let page: Envelope =
        serde_json::from_slice(body).map_err(|_| "Unrecognized worker status response")?;
    if page.instance_id != instance {
        return Err("Relay restarted during this refresh; waiting for a consistent status");
    }
    if let Some(id) = worker {
        let selected = page
            .worker
            .filter(|w| w.worker_id == id)
            .ok_or("Worker identity did not match the selected connection")?;
        Ok(vec![selected])
    } else {
        page.workers.ok_or("Worker inventory was not reported")
    }
}

#[cfg(test)]
#[path = "../../tests/coverage/visualizer/status_tests.rs"]
mod tests;
