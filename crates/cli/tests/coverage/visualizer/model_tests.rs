// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn counts_preserve_integer_precision_and_saturation() {
    assert_eq!(
        number(DiagnosticCount {
            value: 9_007_199_254_740_993.into(),
            saturated: false
        }),
        "9,007,199,254,740,993"
    );
    assert_eq!(
        number(DiagnosticCount {
            value: u64::MAX.into(),
            saturated: true
        }),
        "18,446,744,073,709,551,615+"
    );
    assert_eq!(number(DiagnosticCount::default()), "0");
}

#[test]
fn labels_cannot_emit_terminal_controls_and_have_bounded_length() {
    let sanitized = label(&format!("\x1b[31mheader\n\t{}", "x".repeat(500)));
    assert!(!sanitized.chars().any(char::is_control));
    assert_eq!(sanitized.chars().count(), 120);
}

#[test]
fn new_doctor_checks_flow_through_without_copying_restricted_details() {
    let checks = vec![
        Check {
            name: "Existing check",
            status: Status::Pass,
            details: "secret-old".into(),
        },
        Check {
            name: "Future check",
            status: Status::Warn,
            details: "secret-new".into(),
        },
    ];
    let view = rows(&checks);
    assert_eq!(view.len(), 2);
    assert_eq!(view[1].name, "Future check");
    assert_eq!(view[1].status, Status::Warn);
}

#[test]
fn doctor_adapter_carries_new_observability_checks_to_the_existing_section() {
    use crate::diagnostics::*;
    let layer = || ConfigLayer {
        path: "/unused".into(),
        status: Status::Info,
        active: false,
        details: String::new(),
    };
    let check = || Check {
        name: "Resolution",
        status: Status::Pass,
        details: "not rendered".into(),
    };
    let mut report = DoctorReport {
        schema_version: 2,
        binary_version: "test",
        target_agent: None,
        environment: EnvironmentInfo {
            os: "test".into(),
            arch: "test",
            shell: None,
        },
        configuration: ConfigurationInfo {
            explicit: None,
            global: layer(),
            system: layer(),
            unsupported_project_files: vec![],
            upstream_auth: UpstreamAuthInfo {
                openai: SecretPresence::Unset,
                anthropic: SecretPresence::Unset,
                checks: vec![],
            },
            plugin_configs: vec![],
            plugin_resolution: check(),
            plugin_host_validation: PluginHostValidation {
                status: Status::Info,
                details: String::new(),
                report: None,
            },
            resolution: check(),
            default_agent: None,
            configured_agents: vec![],
            dynamic_plugins: vec![],
        },
        agents: vec![],
        host_plugins: vec![],
        observability: vec![],
        completions: vec![],
    };
    assert!(SetupView::from(&report).sections[2].rows.is_empty());
    report.observability.push(Check {
        name: "New upstream capability",
        status: Status::Warn,
        details: "secret not projected".into(),
    });
    let view = SetupView::from(&report);
    assert_eq!(view.sections[2].rows.len(), 1);
    assert_eq!(view.sections[2].rows[0].name, "New upstream capability");
    assert_eq!(view.sections[2].rows[0].status, Status::Warn);
    assert_eq!(view.schema_version, 2);
}
