// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn app() -> App {
    App::new(&SnapshotSource {
        url: "http://127.0.0.1:47632".into(),
        daemon: false,
        worker: None,
        max_bytes: 131072,
        demo: true,
    })
}

#[test]
fn refresh_controls_are_session_local_and_bounded() {
    let mut app = app();
    assert!(app.automatic);
    assert_eq!(app.interval(), Duration::from_secs(2));
    app.key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE), 40);
    assert!(!app.automatic);
    for _ in 0..10 {
        app.key(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE), 40);
    }
    assert_eq!(app.interval(), Duration::from_secs(10));
    for _ in 0..10 {
        app.key(KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE), 40);
    }
    assert_eq!(app.interval(), Duration::from_secs(1));
    assert_eq!(
        app.key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE), 40),
        (false, true)
    );
    assert_eq!(
        app.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL), 40),
        (true, false)
    );
}

#[test]
fn replacement_snapshot_never_merges_incarnations_and_auth_failure_clears_data() {
    let mut app = app();
    app.accept(Ok(SnapshotRead {
        snapshot: demo::snapshot(0),
        cache: None,
    }));
    app.component_scroll = 5;
    let mut replacement = demo::snapshot(1);
    replacement.source.instance_id = uuid::Uuid::nil();
    replacement.components.clear();
    app.accept(Ok(SnapshotRead {
        snapshot: replacement,
        cache: None,
    }));
    assert_eq!(app.component_scroll, 0);
    assert!(
        app.snapshot
            .as_ref()
            .unwrap()
            .snapshot
            .components
            .is_empty()
    );
    app.accept(Err(ReadError {
        message: "forbidden".into(),
        discard: true,
        retryable: false,
    }));
    assert!(app.snapshot.is_none());
    assert!(app.received_at.is_none());
    assert!(app.automatic);
}

#[test]
fn transient_failure_keeps_only_the_last_snapshot_with_explicit_error() {
    let mut app = app();
    app.accept(Ok(SnapshotRead {
        snapshot: demo::snapshot(0),
        cache: None,
    }));
    app.accept(Err(ReadError {
        message: "busy".into(),
        discard: false,
        retryable: true,
    }));
    assert!(app.snapshot.is_some());
    assert_eq!(app.last_error.as_deref(), Some("busy"));
    assert!(app.automatic);
}

#[test]
fn independent_status_survives_diagnostic_denial_but_never_stays_current_on_failure() {
    let mut app = app();
    app.accept_frame(FrameRead {
        status: Ok(demo::status(0)),
        sampled_at: Instant::now(),
        diagnostics: Err(ReadError {
            message: "forbidden".into(),
            discard: true,
            retryable: false,
        }),
    });
    assert_eq!(app.runtime.as_ref().unwrap().requests(), Some(1));
    assert!(app.automatic);
    assert!(app.snapshot.is_none());
    app.accept_frame(FrameRead {
        status: Err("Cannot reach Relay"),
        sampled_at: Instant::now(),
        diagnostics: Ok(SnapshotRead {
            snapshot: demo::snapshot(0),
            cache: None,
        }),
    });
    assert!(app.runtime.is_none());
    assert!(app.status_at.is_none());
    assert!(app.snapshot.is_some());
}

#[test]
fn navigation_never_launches_doctor_and_setup_requires_explicit_action() {
    let mut app = app();
    let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
    app.key(key(KeyCode::Tab), 40);
    assert!(app.details);
    assert!(!app.setup_requested);
    assert_eq!(app.key(key(KeyCode::Esc), 40), (false, false));
    assert!(!app.details);
    app.key(key(KeyCode::Char('s')), 40);
    assert!(app.setup_tab && app.setup_requested);
    app.setup_requested = false;
    app.setup_running = true;
    app.key(key(KeyCode::Char('s')), 40);
    assert!(!app.setup_requested);
    assert_eq!(app.key(key(KeyCode::Esc), 40), (false, false));
    assert_eq!(app.key(key(KeyCode::Esc), 40), (true, false));
}
