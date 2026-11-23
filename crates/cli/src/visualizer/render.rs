// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Terminal presentation only. Every runtime value comes from the latest canonical page.

use super::{
    App, CADENCES,
    model::{availability, label, number, status},
};
use crate::diagnostics::Status;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table, Widget, Wrap},
};

mod overview;

const BG: Color = Color::Rgb(10, 16, 23);
const FG: Color = Color::Rgb(216, 228, 237);
const MUTED: Color = Color::Rgb(132, 154, 173);
const BORDER: Color = Color::Rgb(55, 78, 96);
const GREEN: Color = Color::Rgb(166, 255, 90);
const CYAN: Color = Color::Rgb(110, 218, 226);
const AMBER: Color = Color::Rgb(246, 195, 106);
const RED: Color = Color::Rgb(255, 108, 134);
const PURPLE: Color = Color::Rgb(185, 177, 255);
const CARD_HEIGHT: u16 = 8;
const HERO_HEIGHT: u16 = CARD_HEIGHT * 2;

fn text(value: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(value.into(), Style::default().fg(color))
}

fn row(name: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![text(format!("{name}  "), MUTED), text(value, FG)])
}

fn panel(title: &str, area: Rect, buf: &mut Buffer) -> Rect {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(text(format!(" {title} "), CYAN)));
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}

fn paragraph(lines: Vec<Line<'static>>, area: Rect, buf: &mut Buffer) {
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(area, buf);
}

pub(super) fn board_height(width: u16) -> u16 {
    let cols = columns(width);
    HERO_HEIGHT + 1 + 6_u16.div_ceil(cols) * (CARD_HEIGHT + 1) - 1
}

fn columns(width: u16) -> u16 {
    if width >= 100 {
        3
    } else if width >= 65 {
        2
    } else {
        1
    }
}

// Keep fixed terminal chrome separate from the scrolling board composition.
fn draw_header(frame: &mut Frame<'_>, app: &App, area: Rect) {
    let mode = if app.automatic {
        format!("AUTO / {}s", CADENCES[app.cadence])
    } else {
        format!("PAUSED / {}s", CADENCES[app.cadence])
    };
    let state = if app.reading {
        "REFRESHING"
    } else if app.status_error.is_some() {
        "STATUS UNAVAILABLE"
    } else if app.runtime.is_some() {
        "SNAPSHOT"
    } else {
        "WAITING"
    };
    let title = Line::from(vec![
        text(" NVIDIA / ", GREEN),
        text("RELAY VISUALIZER", FG),
        text(if app.demo { "  [DEMO]" } else { "  [PoC]" }, AMBER),
        text(
            format!("   {mode}   {state}"),
            if app.status_error.is_some() {
                AMBER
            } else {
                CYAN
            },
        ),
    ]);
    let tabs = Line::from(vec![
        text(
            if !app.setup_tab && !app.details {
                " [Overview] "
            } else {
                "  Overview  "
            },
            GREEN,
        ),
        text(
            if app.details {
                " [Details] "
            } else {
                "  d Details  "
            },
            if app.details { GREEN } else { MUTED },
        ),
        text(
            if app.setup_tab {
                " [Check setup] "
            } else {
                "  s Check setup  "
            },
            if app.setup_tab { GREEN } else { MUTED },
        ),
        text("  Esc back", MUTED),
    ]);
    frame.render_widget(
        Paragraph::new(vec![
            title,
            tabs,
            Line::from(text(label(&app.source_label), MUTED)),
        ]),
        Rect::new(0, 0, area.width, 3),
    );
}

pub(super) fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG).fg(FG)), area);
    if area.width < 40 || area.height < 12 {
        frame.render_widget(
            Paragraph::new("Relay Visualizer\nResize to at least 40 x 12.\nq / Esc to exit")
                .style(Style::default().fg(CYAN)),
            area,
        );
        return;
    }
    draw_header(frame, app, area);

    let viewport = Rect::new(0, 4, area.width, area.height.saturating_sub(7));
    let advanced = app.details || app.setup_tab;
    let total = if advanced {
        board_height(viewport.width)
    } else {
        overview::height(viewport.width)
    };
    app.scroll = app.scroll.min(total.saturating_sub(viewport.height));
    // Render a small virtual board and crop it. All cards keep identical dimensions;
    // scrolling changes the viewport rather than squeezing/truncating modules to fit.
    let mut board = Buffer::empty(Rect::new(0, 0, viewport.width, total));
    board.set_style(board.area, Style::default().bg(BG).fg(FG));
    if !advanced {
        overview::draw(app, &mut board);
    } else {
        let hero = Rect::new(0, 0, viewport.width, HERO_HEIGHT);
        if app.setup_tab {
            setup_overview(app, hero, &mut board);
        } else {
            runtime_overview(app, hero, &mut board);
        }
        paragraph(
            vec![Line::from(text(
                if app.setup_tab {
                    " LOCAL OFFLINE REPORT / explicit action / no automatic probes"
                } else {
                    " SNAPSHOT PANELS / source lifetime / no history"
                },
                MUTED,
            ))],
            Rect::new(0, HERO_HEIGHT, viewport.width, 1),
            &mut board,
        );

        let cols = columns(viewport.width);
        let width = (viewport.width - (cols - 1)) / cols;
        for index in 0..6_u16 {
            let col = index % cols;
            let x = col * (width + 1);
            let w = if col == cols - 1 {
                viewport.width - x
            } else {
                width
            };
            let card = Rect::new(
                x,
                HERO_HEIGHT + 1 + (index / cols) * (CARD_HEIGHT + 1),
                w,
                CARD_HEIGHT,
            );
            if app.setup_tab {
                setup_card(app, index as usize, card, &mut board);
            } else {
                runtime_card(app, index as usize, card, &mut board);
            }
        }
    }
    for y in 0..viewport.height.min(total.saturating_sub(app.scroll)) {
        for x in 0..viewport.width {
            frame.buffer_mut()[(viewport.x + x, viewport.y + y)] =
                board[(x, app.scroll + y)].clone();
        }
    }
    let footer = Rect::new(0, area.height - 3, area.width, 3);
    let hint = if app.setup_tab {
        "q exit | Esc overview | s rerun setup | arrows scroll | j/k check rows"
    } else if app.details {
        "q exit | Esc overview | r refresh | arrows scroll | j/k component rows"
    } else {
        "q exit | Space pause | +/- cadence | r refresh | d details | s check setup"
    };
    let note = if app.details && app.last_error.is_some() {
        format!(
            "Diagnostic evidence not current: {} / r retry",
            app.last_error.as_deref().unwrap_or_default()
        )
    } else {
        format!(
            "Snapshot only / no history. arrows scroll | page {}-{} / {total}",
            app.scroll + 1,
            (app.scroll + viewport.height).min(total)
        )
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(text(hint, GREEN)),
            Line::from(text(
                note,
                if app.last_error.is_some() {
                    AMBER
                } else {
                    MUTED
                },
            )),
            Line::from(text(
                if app.demo {
                    "Illustrative values. No Relay telemetry is collected or sent."
                } else {
                    "Read-only attachment. Relay continues when this viewer exits."
                },
                MUTED,
            )),
        ]),
        footer,
    );
}

fn runtime_overview(app: &mut App, area: Rect, buf: &mut Buffer) {
    let inner = panel("COMPONENT CENSUS", area, buf);
    let Some(read) = &app.snapshot else {
        paragraph(
            vec![
                Line::from(text("Waiting for an authorized diagnostic snapshot", CYAN)),
                Line::from(text(
                    app.last_error
                        .as_deref()
                        .unwrap_or("No producer evidence has been read yet."),
                    AMBER,
                )),
                Line::from(text(
                    "The viewer never starts or recovers a listener.",
                    MUTED,
                )),
            ],
            inner,
            buf,
        );
        return;
    };
    let snapshot = &read.snapshot;
    let source = &snapshot.source;
    paragraph(
        vec![Line::from(text(
            format!(
                " {} / PID {} / source {}",
                label(&source.process_role),
                source
                    .process_id
                    .map_or_else(|| "n/a".into(), |p| p.to_string()),
                source.instance_id
            ),
            MUTED,
        ))],
        Rect::new(inner.x, inner.y, inner.width, 1),
        buf,
    );
    let max = snapshot
        .components
        .iter()
        .map(|c| c.lifetime.totals.recorded_events_total.value.get())
        .max()
        .unwrap_or(0)
        .max(1);
    let capacity = inner.height.saturating_sub(5) as usize;
    app.component_scroll = app
        .component_scroll
        .min(snapshot.components.len().saturating_sub(capacity));
    let bar_width = (inner.width as usize / 4).clamp(1, 32);
    let rows = snapshot
        .components
        .iter()
        .skip(app.component_scroll)
        .take(capacity)
        .map(|component| {
            let totals = component.lifetime.totals;
            let filled = ((totals.recorded_events_total.value.get() as f64 / max as f64)
                * bar_width as f64)
                .round() as usize;
            let bar = format!("{}{}", "▰".repeat(filled), "·".repeat(bar_width - filled));
            let suffix = if component.closed_at.is_some() {
                " [closed]"
            } else {
                ""
            };
            Row::new(vec![
                Cell::from(label(&format!(
                    "{} / g{}{}",
                    component.identity.key,
                    component.identity.generation.get(),
                    suffix
                ))),
                Cell::from(text(bar, GREEN)),
                Cell::from(number(totals.recorded_events_total)),
                Cell::from(text(
                    number(totals.failures_total),
                    if totals.failures_total.value.get() > 0 {
                        RED
                    } else {
                        MUTED
                    },
                )),
            ])
        });
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(34),
            Constraint::Percentage(32),
            Constraint::Percentage(18),
            Constraint::Percentage(16),
        ],
    )
    .header(
        Row::new([
            "COMPONENT / GENERATION",
            "RECORDING CALLS",
            "CALLS",
            "FAILED OCC.",
        ])
        .style(Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
    )
    .column_spacing(1);
    table.render(
        Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(4),
        ),
        buf,
    );
    let rows_note =
        if snapshot.continuation.is_some() || snapshot.coverage.retention.query_truncated {
            "partial page"
        } else {
            "returned page"
        };
    paragraph(
        vec![
            Line::from(text(
                format!(
                    " {} component rows / {rows_note} / j,k scroll rows",
                    snapshot.components.len()
                ),
                MUTED,
            )),
            Line::from(text(
                " Bars compare lifetime diagnostic calls; failed occurrences do not establish current health.",
                MUTED,
            )),
        ],
        Rect::new(inner.x, inner.bottom() - 2, inner.width, 2),
        buf,
    );
}

fn runtime_card(app: &App, index: usize, area: Rect, buf: &mut Buffer) {
    let titles = [
        "01 / OBSERVATIONS",
        "02 / OUTCOME EVIDENCE",
        "03 / SOURCE RETENTION",
        "04 / READ COVERAGE",
        "05 / CAPABILITIES",
        "06 / SOURCE / READER",
    ];
    let inner = panel(titles[index], area, buf);
    let Some(read) = &app.snapshot else {
        paragraph(vec![Line::from(text("No snapshot", MUTED))], inner, buf);
        return;
    };
    let s = &read.snapshot;
    let totals = s.lifetime.totals;
    let lines = match index {
        0 => vec![
            Line::from(
                text(number(totals.recorded_events_total), GREEN)
                    .style(Style::default().fg(GREEN).add_modifier(Modifier::BOLD)),
            ),
            Line::from(text("recording calls", MUTED)),
            row("occurrences", number(totals.occurrences_total)),
            row(
                "source lifetime",
                format!(
                    "{}m {}s",
                    s.lifetime.elapsed_millis.get() / 60_000,
                    (s.lifetime.elapsed_millis.get() / 1000) % 60
                ),
            ),
            Line::from(text("Calls and occurrences differ.", MUTED)),
        ],
        1 => vec![
            row("explicit success", number(totals.successes_total)),
            Line::from(vec![
                text("explicit failure  ", MUTED),
                text(number(totals.failures_total), RED),
            ]),
            row("unknown outcome", number(totals.unspecified_total)),
            Line::from(text("Cumulative recorded evidence.", MUTED)),
            Line::from(text("Recovery is not inferred.", MUTED)),
        ],
        2 => vec![
            row("events in page", s.events.len().to_string()),
            row("events evicted", number(s.coverage.loss.events_evicted)),
            row("events omitted", number(s.coverage.loss.events_omitted)),
            row(
                "components evicted",
                number(s.coverage.loss.components_evicted),
            ),
            Line::from(text("Totals survive event eviction.", MUTED)),
        ],
        3 => vec![
            row("evidence", availability(s.coverage.availability)),
            row(
                "query truncated",
                if s.coverage.retention.query_truncated {
                    "yes"
                } else {
                    "no"
                },
            ),
            row(
                "continuation",
                if s.continuation.is_some() {
                    "more available"
                } else {
                    "none"
                },
            ),
            row(
                "rejected observations",
                number(s.coverage.loss.observations_rejected),
            ),
            Line::from(text("One bounded page per read.", MUTED)),
        ],
        4 => {
            let mut lines = s
                .coverage
                .capabilities
                .iter()
                .take(4)
                .map(|c| row(&label(&c.name), availability(c.availability)))
                .collect::<Vec<_>>();
            if lines.is_empty() {
                lines.push(Line::from(text("No capability entries supplied", MUTED)));
            }
            if s.coverage.capabilities.len() > 4 {
                lines.push(Line::from(text(
                    format!(
                        "+{} more source capabilities",
                        s.coverage.capabilities.len() - 4
                    ),
                    MUTED,
                )));
            } else {
                lines.push(Line::from(text("Support does not imply activity.", MUTED)));
            }
            lines
        }
        _ => {
            let age = app
                .received_at
                .map(|t| format!("{:.1}s", t.elapsed().as_secs_f64()))
                .unwrap_or_else(|| "n/a".into());
            let mut lines = vec![
                row("last read age", age),
                row("sequence", s.sequence.get().to_string()),
                row(
                    "schema / catalogue",
                    format!("{} / {}", s.schema_version, s.catalogue_version),
                ),
                row("captured UTC", s.captured_at.format("%H:%M:%S").to_string()),
            ];
            if let Some(cache) = &read.cache {
                lines.push(row(
                    "cache",
                    format!(
                        "{} / {}",
                        label(&cache.availability),
                        label(&cache.latest_poll)
                    ),
                ));
                lines.push(row(
                    "cache age at read",
                    cache.age_millis.map_or_else(
                        || "n/a".into(),
                        |ms| format!("{:.1}s", ms.get() as f64 / 1000.0),
                    ),
                ));
            } else {
                let millis = super::source::now()
                    .signed_duration_since(s.captured_at)
                    .num_milliseconds();
                lines.push(row(
                    "capture age*",
                    if millis < 0 {
                        "clock ahead".into()
                    } else {
                        format!("{:.1}s", millis as f64 / 1000.0)
                    },
                ));
                lines.push(Line::from(text("* Local wall clock estimate", MUTED)));
            }
            lines
        }
    };
    paragraph(lines, inner, buf);
}

fn status_color(value: Status) -> Color {
    match value {
        Status::Pass => GREEN,
        Status::Warn => AMBER,
        Status::Fail => RED,
        Status::Info => PURPLE,
    }
}

fn setup_overview(app: &App, area: Rect, buf: &mut Buffer) {
    let inner = panel("CHECK SETUP / LOCAL OFFLINE DOCTOR", area, buf);
    let Some(setup) = &app.setup else {
        paragraph(
            vec![
                Line::from(text(
                    app.setup_error
                        .as_deref()
                        .unwrap_or("Checking local setup..."),
                    AMBER,
                )),
                Line::from(text(
                    format!(
                        "Checking local account: {}. This is not the selected worker's setup.",
                        app.local_account
                    ),
                    MUTED,
                )),
            ],
            inner,
            buf,
        );
        return;
    };
    let rows = setup.sections.iter().map(|section| {
        let count = |status| {
            section
                .rows
                .iter()
                .filter(|r| r.status == status)
                .count()
                .to_string()
        };
        Row::new(vec![
            Cell::from(section.title),
            Cell::from(text(count(Status::Pass), GREEN)),
            Cell::from(text(count(Status::Warn), AMBER)),
            Cell::from(text(count(Status::Fail), RED)),
            Cell::from(text(count(Status::Info), PURPLE)),
        ])
    });
    paragraph(
        vec![Line::from(text(
            format!(
                " Account: {} / {} / Doctor schema {} / read {:.0}s ago",
                app.local_account,
                setup.environment,
                setup.schema_version,
                app.setup_at.map_or(0.0, |t| t.elapsed().as_secs_f64())
            ),
            MUTED,
        ))],
        Rect::new(inner.x, inner.y, inner.width, 1),
        buf,
    );
    Table::new(
        rows,
        [
            Constraint::Percentage(48),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
            Constraint::Percentage(13),
        ],
    )
    .header(
        Row::new(["CHECK GROUP", "PASS", "WARN", "FAIL", "INFO"]).style(Style::default().fg(CYAN)),
    )
    .render(
        Rect::new(inner.x, inner.y + 2, inner.width, inner.height - 4),
        buf,
    );
    paragraph(
        vec![
            Line::from(text(
                " Local offline setup only. No remote deployment health claim.",
                MUTED,
            )),
            Line::from(text(
                " Press s to rerun. Runtime refresh never runs Doctor checks.",
                MUTED,
            )),
        ],
        Rect::new(inner.x, inner.bottom() - 2, inner.width, 2),
        buf,
    );
}

fn setup_card(app: &App, index: usize, area: Rect, buf: &mut Buffer) {
    let Some(setup) = &app.setup else {
        panel("LOCAL SETUP", area, buf);
        return;
    };
    let section = &setup.sections[index];
    let inner = panel(section.title, area, buf);
    let capacity = inner.height.saturating_sub(1) as usize;
    let start = app
        .component_scroll
        .min(section.rows.len().saturating_sub(capacity));
    let mut lines = section
        .rows
        .iter()
        .skip(start)
        .take(capacity)
        .map(|r| {
            Line::from(vec![
                text(format!("{} ", status(r.status)), status_color(r.status)),
                text(&r.name, FG),
            ])
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push(Line::from(text("No checks reported", MUTED)));
    }
    if section.rows.len() > capacity {
        lines.push(Line::from(text(
            format!("{} checks / j,k scroll", section.rows.len()),
            MUTED,
        )));
    }
    paragraph(lines, inner, buf);
}

#[cfg(test)]
#[path = "../../tests/coverage/visualizer/render_tests.rs"]
mod tests;
