// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::visualizer::{
    demo,
    source::{SnapshotRead, SnapshotSource},
};
use ratatui::{Terminal, backend::TestBackend};

fn app() -> App {
    let mut app = App::new(&SnapshotSource {
        url: String::new(),
        daemon: false,
        worker: None,
        max_bytes: 131072,
        demo: true,
    });
    app.accept(Ok(SnapshotRead {
        snapshot: demo::snapshot(0),
        cache: None,
    }));
    app.runtime = Some(demo::status(0));
    app.status_at = Some(std::time::Instant::now());
    app
}

fn screen(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

#[test]
fn technical_evidence_remains_available_under_details() {
    let mut app = app();
    app.details = true;
    let image = screen(&mut app, 150, 44);
    for value in [
        "RELAY VISUALIZER",
        "[DEMO]",
        "COMPONENT CENSUS",
        "1,284",
        "1,324",
        "OUTCOME EVIDENCE",
        "SOURCE RETENTION",
        "unsupported",
        "SOURCE / READER",
    ] {
        assert!(image.contains(value), "missing {value}");
    }
    assert!(!image.contains("GPU"));
    assert!(!image.contains("Timeline"));
}

#[test]
fn partial_coverage_cannot_be_presented_as_complete() {
    let mut app = app();
    app.details = true;
    app.snapshot
        .as_mut()
        .unwrap()
        .snapshot
        .coverage
        .retention
        .query_truncated = true;
    let image = screen(&mut app, 150, 44);
    assert!(image.contains("partial page"));
    assert!(image.contains("query truncated  yes"));
}

#[test]
fn resize_and_scrolling_do_not_panic_or_lose_exit_controls() {
    for (width, height) in [(1, 1), (40, 12), (64, 24), (80, 24), (100, 30), (150, 44)] {
        let mut app = app();
        app.scroll = u16::MAX;
        app.component_scroll = usize::MAX;
        let image = screen(&mut app, width, height);
        assert!(!image.is_empty());
        if width >= 40 && height >= 12 {
            assert!(image.contains("q exit"));
        }
    }
}

#[test]
fn overview_answers_questions_without_promoting_recording_counts_to_work() {
    let image = screen(&mut app(), 150, 36);
    for value in [
        "AM I CONNECTED?",
        "IS WORK PASSING THROUGH?",
        "WHAT NEEDS ATTENTION?",
        "requests in flight",
        "Connected sessions",
        "Earlier failures recorded: 3",
        "Recovery unknown",
        "[DEMO]",
    ] {
        assert!(image.contains(value), "missing {value}");
    }
    for value in [
        "1,284",
        "1,324",
        "COMPONENT CENSUS",
        "SOURCE RETENTION",
        "exporter broken",
    ] {
        assert!(!image.contains(value), "unexpected {value}");
    }
}

#[test]
fn missing_activity_cannot_render_as_idle_or_healthy() {
    let mut app = app();
    app.runtime = None;
    app.status_error = Some("Cannot reach Relay");
    let image = screen(&mut app, 150, 36);
    assert!(image.contains("NOT REPORTED"));
    assert!(!image.contains("No connection issue reported"));
    app.runtime = Some(demo::status(2));
    app.status_error = None;
    let image = screen(&mut app, 150, 36);
    assert!(image.contains("local work"));
    assert!(image.contains("No connection issue reported"));
    assert!(!image.contains("agent idle"));
}
