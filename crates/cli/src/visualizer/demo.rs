// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Explicit, deterministic illustrative evidence; never used as a fallback for live reads.

use super::source::now;
use nemo_relay_types::diagnostics::*;
use uuid::Uuid;

fn count(value: u64) -> DiagnosticCount {
    DiagnosticCount {
        value: value.into(),
        saturated: false,
    }
}

pub(super) fn snapshot(frame: u64) -> DiagnosticSnapshot {
    let captured_at = now();
    let elapsed = 1_122_000 + frame * 2_000;
    let mut total = DiagnosticTotals::default();
    let components = [
        ("cli.gateway", 480, 510, 2, 12),
        ("logging", 360, 360, 0, 0),
        ("plugin.activation", 240, 244, 1, 0),
        ("daemon.worker", 120, 126, 0, 8),
        ("configuration", 60, 60, 0, 3),
        ("diagnostics", 24, 24, 0, 2),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (name, calls, occurrences, failures, unknown))| {
        let increment = if i == 0 { frame * 12 } else { 0 };
        let calls = calls + increment;
        let occurrences = occurrences + increment;
        let successes = occurrences - failures - unknown;
        total.recorded_events_total.add(calls);
        total.occurrences_total.add(occurrences);
        total.successes_total.add(successes);
        total.failures_total.add(failures);
        total.unspecified_total.add(unknown);
        let totals = DiagnosticTotals {
            recorded_events_total: count(calls),
            occurrences_total: count(occurrences),
            successes_total: count(successes),
            failures_total: count(failures),
            unspecified_total: count(unknown),
            ..Default::default()
        };
        ComponentSnapshot {
            identity: ComponentIdentity {
                key: name.into(),
                instance_id: Uuid::from_u128(i as u128 + 100),
                generation: 1.into(),
            },
            registered_at: captured_at - std::time::Duration::from_millis(elapsed),
            closed_at: None,
            lifetime: DiagnosticLifetimeSummary {
                totals,
                elapsed_millis: elapsed.into(),
                ..Default::default()
            },
            since_refresh: DiagnosticRefreshSummary {
                generation: 1.into(),
                started_at: captured_at - std::time::Duration::from_millis(elapsed),
                elapsed_millis: elapsed.into(),
                ownership: Default::default(),
                totals,
            },
            counters: vec![],
            windows: vec![],
        }
    })
    .collect::<Vec<_>>();
    DiagnosticSnapshot {
        schema_version: DIAGNOSTIC_SCHEMA_VERSION,
        catalogue_version: DIAGNOSTIC_CATALOGUE_VERSION,
        source: SourceIdentity {
            instance_id: Uuid::from_u128(0x7b2c_0000_0000_0000_0000_0000_0000_3a9f),
            process_id: None,
            process_role: "illustrative gateway".into(),
            relay_version: env!("CARGO_PKG_VERSION").into(),
        },
        captured_at,
        sequence: (9842 + frame * 12).into(),
        lifetime: DiagnosticLifetimeSummary {
            totals: total,
            elapsed_millis: elapsed.into(),
            ..Default::default()
        },
        coverage: DiagnosticCoverage {
            retention: DiagnosticRetention {
                components_retained: (components.len() as u64).into(),
                ..Default::default()
            },
            capabilities: vec![
                DiagnosticCapability {
                    name: "cli".into(),
                    availability: DiagnosticAvailability::Available,
                    reason: None,
                },
                DiagnosticCapability {
                    name: "logging".into(),
                    availability: DiagnosticAvailability::Available,
                    reason: None,
                },
                DiagnosticCapability {
                    name: "otel".into(),
                    availability: DiagnosticAvailability::Unsupported,
                    reason: Some("not_integrated".into()),
                },
            ],
            ..Default::default()
        },
        components,
        events: vec![],
        continuation: None,
    }
}

/// Explicitly synthetic transport samples; diagnostic totals are not activity counters.
pub(super) fn status(frame: u64) -> super::status::RuntimeStatus {
    use super::status::{Health, RuntimeStatus, WorkerStatus};
    let worker = Uuid::from_u128(0x01a1_1234_5678_7000_8000_0000_0000_0001);
    RuntimeStatus {
        health: Health {
            service: "nemo-relay-daemon".into(),
            status: "ok".into(),
            version: "0.11.0".into(),
            instance_id: "demo-daemon".into(),
            deployment_mode: Some("managed".into()),
        },
        workers: Some(vec![WorkerStatus {
            worker_id: worker,
            state: "ready".into(),
            reference_count: 2,
            control_available: true,
            in_flight: [1, 2, 0, 1][frame as usize % 4],
        }]),
        worker_error: None,
        selected_worker: Some(worker),
    }
}
