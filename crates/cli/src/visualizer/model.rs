// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Pure report-to-view mapping. No terminal, network, configuration resolution or probes.

use crate::diagnostics::{Check, DoctorReport, Status};
use nemo_relay_types::diagnostics::{DiagnosticAvailability, DiagnosticCount};

/// Do not allow producer labels to inject terminal controls or unbounded single-line content.
/// Restricted event messages, caller correlation and credentials are never mapped into this view.
pub(super) fn label(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(120)
        .collect()
}

/// Include the effective Unix UID because a privileged demo launcher may run the
/// viewer as the daemon owner rather than the person logged into the terminal.
pub(super) fn local_account() -> String {
    let name = label(
        &std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown account".into()),
    );
    #[cfg(unix)]
    {
        // SAFETY: geteuid has no preconditions and cannot fail.
        let uid = unsafe { libc::geteuid() };
        format!("{name} (UID {uid})")
    }
    #[cfg(not(unix))]
    name
}

pub(super) fn number(value: DiagnosticCount) -> String {
    let digits = value.value.get().to_string();
    let mut result = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    if value.saturated {
        result.push('+');
    }
    result
}

pub(super) fn availability(value: DiagnosticAvailability) -> &'static str {
    match value {
        DiagnosticAvailability::Available => "available",
        DiagnosticAvailability::Unavailable => "unavailable",
        DiagnosticAvailability::Unsupported => "unsupported",
        DiagnosticAvailability::NeverObserved => "never observed",
        DiagnosticAvailability::Disabled => "disabled",
    }
}

pub(super) fn status(value: Status) -> &'static str {
    match value {
        Status::Pass => "PASS",
        Status::Warn => "WARN",
        Status::Fail => "FAIL",
        Status::Info => "INFO",
    }
}

pub(super) struct SetupRow {
    pub name: String,
    pub status: Status,
}

pub(super) struct SetupSection {
    pub title: &'static str,
    pub rows: Vec<SetupRow>,
}

pub(super) struct SetupView {
    pub schema_version: u32,
    pub sections: Vec<SetupSection>,
    pub environment: String,
}

fn rows(checks: &[Check]) -> Vec<SetupRow> {
    checks
        .iter()
        .map(|check| SetupRow {
            name: label(check.name),
            status: check.status,
        })
        .collect()
}

impl From<&DoctorReport> for SetupView {
    fn from(report: &DoctorReport) -> Self {
        let mut configuration = rows(&[
            report.configuration.resolution.clone(),
            report.configuration.plugin_resolution.clone(),
        ]);
        configuration.push(SetupRow {
            name: "Plugin host validation".into(),
            status: report.configuration.plugin_host_validation.status,
        });
        // New checks in existing collections flow through without widget-specific changes.
        let mut agents = Vec::new();
        for agent in &report.agents {
            agents.push(SetupRow {
                name: label(agent.name),
                status: agent.status,
            });
            agents.extend(rows(&agent.checks));
        }
        let plugins = report
            .host_plugins
            .iter()
            .flat_map(|plugin| {
                plugin.checks.iter().map(|check| SetupRow {
                    name: label(&format!("{} / {}", plugin.host, check.name)),
                    status: if check.ok { Status::Pass } else { Status::Fail },
                })
            })
            .collect();
        Self {
            schema_version: report.schema_version,
            environment: label(&format!(
                "{} / {} / {}",
                report.environment.os, report.environment.arch, report.binary_version
            )),
            sections: vec![
                SetupSection {
                    title: "CONFIGURATION",
                    rows: configuration,
                },
                SetupSection {
                    title: "AGENT READINESS",
                    rows: agents,
                },
                SetupSection {
                    title: "OBSERVABILITY",
                    rows: rows(&report.observability),
                },
                SetupSection {
                    title: "HOST PLUGINS",
                    rows: plugins,
                },
                SetupSection {
                    title: "AUTH REFERENCES",
                    rows: rows(&report.configuration.upstream_auth.checks),
                },
                SetupSection {
                    title: "COMPLETIONS",
                    rows: rows(&report.completions),
                },
            ],
        }
    }
}

#[cfg(test)]
#[path = "../../tests/coverage/visualizer/model_tests.rs"]
mod tests;
