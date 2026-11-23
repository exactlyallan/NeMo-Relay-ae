// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use serde_json::json;

#[test]
fn status_requires_correct_service_and_consistent_instance() {
    let health = br#"{"service":"nemo-relay-daemon","status":"ok","version":"0.11.0","instance_id":"first"}"#;
    assert!(decode_health(200, health, true).is_ok());
    assert!(decode_health(200, health, false).is_err());
    assert!(decode_health(503, health, true).is_err());
    let workers = br#"{"instance_id":"second","workers":[]}"#;
    assert!(
        decode_workers(200, workers, "first", None)
            .unwrap_err()
            .contains("restarted")
    );
}

#[test]
fn missing_worker_and_wrong_worker_never_become_empty_activity() {
    let id = Uuid::from_u128(7);
    let body = serde_json::to_vec(&json!({
        "instance_id": "first",
        "worker": {"worker_id": Uuid::nil(), "state":"ready", "control_available":true, "in_flight":0, "reference_count":1}
    })).unwrap();
    assert!(decode_workers(200, &body, "first", Some(id)).is_err());
    assert!(
        decode_workers(404, b"", "first", Some(id))
            .unwrap_err()
            .contains("replaced")
    );
    assert!(decode_workers(200, br#"{"instance_id":"first"}"#, "first", None).is_err());
}

#[test]
fn unavailable_and_zero_are_distinct_and_sums_do_not_overflow() {
    let mut status = crate::visualizer::demo::status(2);
    assert_eq!(status.requests(), Some(0));
    assert_eq!(status.sessions(), Some(2));
    status.workers = None;
    assert_eq!(status.requests(), None);
    assert_eq!(status.sessions(), None);
    status.workers = Some(vec![]);
    assert_eq!(status.requests(), Some(0));
    let mut worker = crate::visualizer::demo::status(0)
        .workers
        .unwrap()
        .remove(0);
    worker.in_flight = u64::MAX;
    status.workers = Some(vec![worker.clone(), worker]);
    assert_eq!(status.requests(), None);
}

#[test]
fn public_status_client_has_narrow_routes_and_no_authorization_headers() {
    use crate::gateway::client::get_visualizer_status;
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    for path in [
        "/_nemo_relay/v1/diagnostics",
        "/healthz\r\nAuthorization: bad",
        "/_nemo_relay/v1/workers/invalid/status",
        "/healthz?redirect=1",
    ] {
        assert!(get_visualizer_status("http://127.0.0.1:1", path).is_err());
    }
    assert!(get_visualizer_status("http://192.0.2.1:80", "/healthz").is_err());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut request = vec![];
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            socket.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
        String::from_utf8(request).unwrap()
    });
    let response = get_visualizer_status(&url, "/healthz").unwrap();
    assert_eq!(response.body, b"{}");
    assert_eq!(
        server.join().unwrap(),
        format!(
            "GET /healthz HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            url.trim_start_matches("http://")
        )
    );
}
