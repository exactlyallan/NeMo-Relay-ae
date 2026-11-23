// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Question-led presentation. Recorder occupancy and observation totals are details,
//! never proxies for agent work or end-to-end health.

use super::*;

const SUMMARY: u16 = 7;
const TILE: u16 = 16;

pub(super) fn height(width: u16) -> u16 {
    SUMMARY + 1 + 3_u16.div_ceil(columns(width)) * (TILE + 1) + 3
}

pub(super) fn draw(app: &App, buf: &mut Buffer) {
    let width = buf.area.width;
    let inner = panel("CURRENT SNAPSHOT", Rect::new(0, 0, width, SUMMARY), buf);
    let (headline, color) = match &app.runtime {
        Some(status) if status.worker_error.is_some() => {
            ("Relay reachable / worker status unavailable", AMBER)
        }
        Some(_) => (
            "Relay reachable / inspect connection and activity below",
            GREEN,
        ),
        None if app.status_error.is_some() => ("Relay status unavailable", AMBER),
        None => ("Connecting to the selected Relay instance...", CYAN),
    };
    let age = app.status_at.map_or_else(
        || "not sampled".into(),
        |t| format!("sampled {:.1}s ago", t.elapsed().as_secs_f64()),
    );
    paragraph(
        vec![
            Line::from(text(headline, color)),
            row(
                "Status",
                format!(
                    "{age} / {}",
                    if app.automatic {
                        "automatic refresh"
                    } else {
                        "polling paused"
                    }
                ),
            ),
            Line::from(text("Session -> Relay -> upstream provider", CYAN)),
            Line::from(text(
                "Local connection status only. Provider delivery and agent progress are not checked.",
                MUTED,
            )),
        ],
        inner,
        buf,
    );
    let cols = columns(width);
    let tile_width = (width - (cols - 1)) / cols;
    for index in 0..3_u16 {
        let col = index % cols;
        let x = col * (tile_width + 1);
        let area = Rect::new(
            x,
            SUMMARY + 1 + index / cols * (TILE + 1),
            if col == cols - 1 {
                width - x
            } else {
                tile_width
            },
            TILE,
        );
        match index {
            0 => connection(app, area, buf),
            1 => activity(app, area, buf),
            _ => problems(app, area, buf),
        }
    }
    paragraph(
        vec![
            Line::from(text(
                "Coverage: model calls, tool calls, tokens, cost and export delivery are not reported by these sources.",
                MUTED,
            )),
            Line::from(text(
                "d Details: diagnostic evidence / s Check setup: run local Doctor explicitly",
                CYAN,
            )),
        ],
        Rect::new(0, height(width) - 3, width, 3),
        buf,
    );
}

fn connection(app: &App, area: Rect, buf: &mut Buffer) {
    let inner = panel("01 / AM I CONNECTED?", area, buf);
    let Some(runtime) = &app.runtime else {
        paragraph(
            vec![
                Line::from(text("NOT REPORTED", AMBER)),
                Line::from(text(
                    app.status_error.unwrap_or("Waiting for a status response"),
                    FG,
                )),
                Line::from(text(
                    "Check the selected listener address and whether Relay is running. Press r to retry.",
                    MUTED,
                )),
            ],
            inner,
            buf,
        );
        return;
    };
    let mut lines = vec![
        row(
            "Relay",
            format!(
                "{} / {}",
                label(&runtime.health.version),
                runtime
                    .health
                    .deployment_mode
                    .as_deref()
                    .map(label)
                    .unwrap_or_else(|| "gateway".into())
            ),
        ),
        row(
            "Scope",
            if runtime.selected_worker.is_some() {
                "selected worker"
            } else if runtime.workers.is_some() {
                "all reported workers"
            } else {
                "gateway"
            },
        ),
    ];
    if let Some(workers) = &runtime.workers {
        let ready = workers.iter().filter(|w| w.state == "ready").count();
        let controlled = workers.iter().filter(|w| w.control_available).count();
        lines.extend([
            row("Workers ready", format!("{ready} / {}", workers.len())),
            row(
                "Control connected",
                format!("{controlled} / {}", workers.len()),
            ),
            row(
                "Connected sessions",
                runtime
                    .sessions()
                    .map_or_else(|| "Not reported".into(), |v| v.to_string()),
            ),
            Line::from(text(
                "Sessions are connections, not a user or agent count.",
                MUTED,
            )),
        ]);
        if let Some(worker) = runtime.selected_worker {
            lines.push(row("Worker", worker.to_string()));
        }
        if workers.is_empty() {
            lines.push(Line::from(text(
                "Open a Relay-enabled agent session to assign a worker.",
                CYAN,
            )));
        }
    } else {
        lines.extend([
            row("Sessions", "Not reported"),
            Line::from(text(
                runtime
                    .worker_error
                    .unwrap_or("This gateway does not expose worker connection status."),
                MUTED,
            )),
        ]);
    }
    paragraph(lines, inner, buf);
}

fn activity(app: &App, area: Rect, buf: &mut Buffer) {
    let inner = panel("02 / IS WORK PASSING THROUGH?", area, buf);
    let requests = app.runtime.as_ref().and_then(|r| r.requests());
    let mut lines = vec![
        Line::from(
            text(
                requests.map_or_else(|| "NOT REPORTED".into(), |n| n.to_string()),
                if requests.is_some() { GREEN } else { AMBER },
            )
            .style(
                Style::default()
                    .fg(if requests.is_some() { GREEN } else { AMBER })
                    .add_modifier(Modifier::BOLD),
            ),
        ),
        Line::from(text("requests in flight at this sample", CYAN)),
        Line::from(""),
        Line::from(text(
            match requests {
                Some(0) => {
                    "No transport request at sample time. The agent may still be doing local work."
                }
                Some(_) => {
                    "Transport requests are active. This does not identify the model, tool or agent task."
                }
                None => "Current request activity is unavailable for this connection.",
            },
            FG,
        )),
        Line::from(""),
        Line::from(text(
            "Short requests can finish between polls. No completed-request total is available.",
            MUTED,
        )),
    ];
    if app
        .runtime
        .as_ref()
        .is_some_and(|r| r.selected_worker.is_none() && r.workers.is_some())
    {
        lines.push(Line::from(text(
            "Count includes all reported workers.",
            AMBER,
        )));
    }
    paragraph(lines, inner, buf);
}

fn problems(app: &App, area: Rect, buf: &mut Buffer) {
    let inner = panel("03 / WHAT NEEDS ATTENTION?", area, buf);
    let mut lines = Vec::new();
    if let Some(reason) = app.status_error {
        lines.push(Line::from(text(reason, AMBER)));
        lines.push(Line::from(text(
            "Check Relay and the selected address; r retries.",
            MUTED,
        )));
    } else if let Some(runtime) = &app.runtime {
        if let Some(reason) = runtime.worker_error {
            lines.push(Line::from(text(reason, AMBER)));
            lines.push(Line::from(text(
                "Refresh or reconnect to the current worker.",
                MUTED,
            )));
        } else if let Some(workers) = &runtime.workers {
            let affected = workers
                .iter()
                .filter(|w| w.state != "ready" || !w.control_available)
                .count();
            if affected > 0 {
                lines.push(Line::from(text(
                    format!("{affected} worker(s) not ready or control disconnected"),
                    AMBER,
                )));
                lines.push(Line::from(text(
                    "Retry; if this persists, check daemon logs.",
                    MUTED,
                )));
            } else {
                lines.push(Line::from(text("No connection issue reported", GREEN)));
            }
        } else {
            lines.push(Line::from(text(
                "Relay reachable; worker checks unavailable",
                MUTED,
            )));
        }
    } else {
        lines.push(Line::from(text("Waiting for connection checks", MUTED)));
    }
    if app.last_error.is_some() {
        lines.push(Line::from(text(
            "Diagnostic evidence not current; d for access/configuration details.",
            AMBER,
        )));
    } else if let Some(read) = &app.snapshot {
        let stale = read
            .cache
            .as_ref()
            .is_some_and(|c| c.availability != "available" && c.availability != "fresh");
        if stale {
            lines.push(Line::from(text(
                "Diagnostic cache not fresh; d for collection details.",
                AMBER,
            )));
        }
        let failures = read.snapshot.lifetime.totals.failures_total;
        if failures.value.get() > 0 || failures.saturated {
            lines.push(Line::from(""));
            lines.push(Line::from(text(
                format!("Earlier failures recorded: {}", number(failures)),
                AMBER,
            )));
            lines.push(Line::from(text(
                "Recovery unknown. These are lifetime records, not current faults. d for evidence.",
                MUTED,
            )));
        }
    }
    paragraph(lines, inner, buf);
}
