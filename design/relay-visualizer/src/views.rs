// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Native Ratatui design fixture. Meter scales are explicit and linear; their
//! ranges are display references, never inferred capacity or alert thresholds.
use crate::App;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Gauge, Paragraph, Tabs, Widget, Wrap},
};

const BG: Color = Color::Rgb(10, 17, 23);
const PANEL: Color = Color::Rgb(14, 24, 32);
const FG: Color = Color::Rgb(219, 232, 239);
const SOFT: Color = Color::Rgb(145, 168, 183);
const EDGE: Color = Color::Rgb(52, 76, 91);
const CYAN: Color = Color::Rgb(108, 214, 224);
const LIME: Color = Color::Rgb(182, 232, 124);
const AMBER: Color = Color::Rgb(245, 193, 110);
const AMBER_BG: Color = Color::Rgb(48, 37, 24);
const VIOLET: Color = Color::Rgb(188, 174, 239);

fn span(s: impl Into<String>, c: Color) -> Span<'static> {
    Span::styled(s.into(), Style::default().fg(c))
}
fn bold(s: impl Into<String>, c: Color) -> Span<'static> {
    Span::styled(
        s.into(),
        Style::default().fg(c).add_modifier(Modifier::BOLD),
    )
}
fn plain(s: impl Into<String>) -> Line<'static> {
    Line::from(span(s, FG))
}
fn muted(s: impl Into<String>) -> Line<'static> {
    Line::from(span(s, SOFT))
}
fn para(lines: Vec<Line<'static>>, area: Rect, b: &mut Buffer) {
    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .render(area, b);
}
fn at(a: Rect, y: u16, h: u16) -> Rect {
    Rect::new(a.x, a.y + y, a.width, h)
}
fn cols(a: Rect, c: impl IntoIterator<Item = Constraint>) -> std::rc::Rc<[Rect]> {
    Layout::horizontal(c).spacing(1).split(a)
}
fn panel(title: &str, a: Rect, b: &mut Buffer, c: Color) -> Rect {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(PANEL))
        .border_style(Style::default().fg(EDGE))
        .title(Line::from(bold(format!(" {title} "), c)));
    let inner = block.inner(a);
    block.render(a, b);
    inner
}
/// All percentage gauges occupy exactly one terminal row, irrespective of caller.
fn gauge(a: Rect, b: &mut Buffer, label: &str, ratio: f64, c: Color) {
    Gauge::default()
        .ratio(ratio)
        .label(Span::styled(
            label.to_owned(),
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .gauge_style(Style::default().fg(c).bg(EDGE))
        .use_unicode(true)
        .render(Rect::new(a.x, a.y, a.width, 1), b);
}
fn center(b: &mut Buffer, a: Rect, y: u16, value: &str, c: Color, heavy: bool) {
    let line = if heavy {
        Line::from(bold(value, c))
    } else {
        Line::from(span(value, c))
    };
    Paragraph::new(line).centered().render(at(a, y, 1), b);
}
fn tick(value: f64) -> String {
    if value >= 1000.0 {
        format!("{}k", (value / 1000.0).round() as u64)
    } else {
        format!("{}", value.round() as u64)
    }
}

/// Calibrated ten-step vertical meter bank: native cells, fixed display range,
/// visible zero and ticks, category labels and exact values. Partial LEDs retain
/// a fractional cell so a value is not silently rounded to a whole segment.
fn equalizer(a: Rect, b: &mut Buffer, data: &[(&str, f64, &str, Color)], max: f64) {
    let axis = 6;
    let plot = Rect::new(a.x + axis, a.y, a.width.saturating_sub(axis), 12);
    let band = plot.width / data.len() as u16;
    let brick = 7.min(band.saturating_sub(2));
    for row in 0..10_u16 {
        let level = 10 - row;
        if row % 2 == 0 {
            let label = format!("{:>3} ┤", tick(max * f64::from(level) / 10.0));
            b.set_string(a.x, a.y + row, label, Style::default().fg(SOFT));
        }
        let low = max * f64::from(level - 1) / 10.0;
        for (i, (_, value, _, color)) in data.iter().enumerate() {
            let fraction = ((*value - low) / (max / 10.0)).clamp(0.0, 1.0);
            let step = (fraction * 6.0).round() as usize;
            let symbol = if fraction <= 0.0 {
                "▆"
            } else {
                ["▁", "▁", "▂", "▃", "▄", "▅", "▆"][step]
            };
            let x = plot.x + i as u16 * band + (band - brick) / 2;
            for dx in 0..brick {
                b[(x + dx, a.y + row)]
                    .set_symbol(symbol)
                    .set_fg(if fraction <= 0.0 { EDGE } else { *color });
            }
        }
    }
    b.set_string(a.x, a.y + 10, "  0 └", Style::default().fg(SOFT));
    for (i, (name, _, value, _)) in data.iter().enumerate() {
        let slot = Rect::new(plot.x + i as u16 * band, plot.y, band, 12);
        center(b, slot, 10, value, FG, true);
        center(b, slot, 11, name, SOFT, false);
    }
}

/// Compact AV-style horizontal meter. Labels and values remain outside the LEDs;
/// the shared 0/mid/max ruler supplies units and context, even for a tiny reading.
fn vu(
    a: Rect,
    b: &mut Buffer,
    data: &[(&str, f64, &str, Color)],
    max: f64,
    axis_max: &str,
    axis_mid: &str,
    unit: &str,
) {
    let label_w = data.iter().map(|d| d.0.len()).max().unwrap_or(0) as u16;
    let value_w = data
        .iter()
        .map(|d| d.2.len())
        .max()
        .unwrap_or(0)
        .max(unit.len()) as u16;
    let x = a.x + label_w + 1;
    let meter_w = a.width.saturating_sub(label_w + value_w + 3);
    for (i, (name, value, text, color)) in data.iter().enumerate() {
        let y = a.y + i as u16;
        b.set_string(a.x, y, name, Style::default().fg(SOFT));
        let amount = (*value / max).clamp(0.0, 1.0) * f64::from(meter_w);
        for j in 0..meter_w {
            let fill = (amount - f64::from(j)).clamp(0.0, 1.0);
            let c = if fill >= 0.5 { *color } else { EDGE };
            b[(x + j, y)].set_symbol("▰").set_fg(c);
        }
        b.set_string(
            x + meter_w + 1,
            y,
            text,
            Style::default().fg(FG).add_modifier(Modifier::BOLD),
        );
    }
    let y = a.bottom() - 1;
    b.set_string(x, y, "0", Style::default().fg(SOFT));
    let middle = x + meter_w / 2;
    b.set_string(
        middle.saturating_sub(axis_mid.len() as u16 / 2),
        y,
        axis_mid,
        Style::default().fg(SOFT),
    );
    b.set_string(
        x + meter_w - axis_max.len() as u16,
        y,
        axis_max,
        Style::default().fg(SOFT),
    );
    b.set_string(x + meter_w + 1, y, unit, Style::default().fg(SOFT));
}

pub(super) fn draw(f: &mut Frame<'_>, app: &mut App) {
    let a = f.area();
    f.render_widget(Block::default().style(Style::default().fg(FG).bg(BG)), a);
    if a.width < 80 || a.height < 24 {
        f.render_widget(Paragraph::new("RELAY VISUALIZER / RATATUI MOCK\nResize to at least 80 x 24 cells.\nTab: pages · q: quit"),a);
        return;
    }
    f.render_widget(
        Paragraph::new(Line::from(vec![
            bold(" NeMo Relay Viz", LIME),
            span(" · v0.11.0 · local · managed daemon · 127.0.0.1:10011", FG),
        ])),
        Rect::new(0, 0, a.width, 1),
    );
    f.render_widget(
        Paragraph::new(muted(format!(
            " q:quit · tab:next-screen · r:refresh · space:refresh-rate [{}]",
            app.rate_label()
        ))),
        Rect::new(0, 1, a.width, 1),
    );
    f.render_widget(
        Paragraph::new(Line::from(span("─".repeat(a.width as usize), EDGE))),
        Rect::new(0, 2, a.width, 1),
    );
    let nav = Rect::new(0, 3, a.width, 1);
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(28, 69, 79))),
        nav,
    );
    f.render_widget(
        Tabs::new([" Overview ", " Observability Plugin ", " Hardware Plugin "])
            .select(app.page)
            .divider(" │ ")
            .style(Style::default().fg(CYAN).bg(Color::Rgb(28, 69, 79)))
            .highlight_style(
                Style::default()
                    .fg(BG)
                    .bg(CYAN)
                    .add_modifier(Modifier::BOLD),
            ),
        nav,
    );
    let wide = a.width >= 100;
    let mut total: u16 = if wide {
        [20, 36, 32][app.page]
    } else {
        [31, 69, 50][app.page]
    };
    if app.details {
        total += 7;
    }
    let viewport = Rect::new(0, 5, a.width, a.height.saturating_sub(8));
    app.scroll = app.scroll.min(total.saturating_sub(viewport.height));
    let mut b = Buffer::empty(Rect::new(0, 0, a.width, total));
    b.set_style(b.area, Style::default().fg(FG).bg(BG));
    match app.page {
        0 => overview(&mut b, app, wide),
        1 => observability(&mut b, app, wide),
        _ => hardware(&mut b, app, wide),
    }
    if app.details {
        let inner = panel(
            " SOURCE DETAILS ",
            Rect::new(0, total - 7, a.width, 7),
            &mut b,
            SOFT,
        );
        let first = match app.page {
            0 => "cli.upstream.response_failed · timeout · 1 occurrence",
            1 => "Event-derived counters since 14:14 UTC; normalized token fields.",
            _ => "Process tree with owned Linux cgroup; per-field sample coverage.",
        };
        para(
            vec![
                muted(first),
                muted("Meter ranges are linear display scales, not capacity or health thresholds."),
                muted("All percentages use a known denominator. Missing fields remain unknown."),
                muted("Fixture data only; no collection, probes, or live connection."),
            ],
            inner,
            &mut b,
        );
    }
    for y in 0..viewport.height.min(total - app.scroll) {
        for x in 0..viewport.width {
            f.buffer_mut()[(x, y + viewport.y)] = b[(x, y + app.scroll)].clone();
        }
    }
    f.render_widget(
        Paragraph::new(vec![
            muted(format!(
                " Runtime snapshot {} UTC · read {} · {}",
                app.time(),
                app.revision + 1,
                if app.rate_seconds().is_none() {
                    "automatic refresh paused"
                } else {
                    "runtime polling"
                }
            )),
            muted(format!(
                " ↑↓:scroll · d:source details · rows {}–{} / {}",
                app.scroll + 1,
                (app.scroll + viewport.height).min(total),
                total,
            )),
            muted(" MOCK · fixture data · no live connection or configuration changes"),
        ]),
        Rect::new(0, a.height - 3, a.width, 3),
    );
}

fn overview(b: &mut Buffer, app: &App, wide: bool) {
    let w = b.area.width;
    let cells = cols(
        Rect::new(0, 0, w, 6),
        [
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ],
    );
    let connection = panel("CONNECTION", cells[0], b, LIME);
    para(
        vec![
            Line::from(bold("● Connected", LIME)),
            plain("1 worker"),
            plain(format!(
                "{} request{} in flight",
                app.requests(),
                if app.requests() == 1 { "" } else { "s" }
            )),
        ],
        connection,
        b,
    );
    let sessions = panel("SESSIONS", cells[1], b, CYAN);
    para(
        vec![
            Line::from(bold("2 connected", FG)),
            plain("Client connections"),
            muted("Selected worker"),
        ],
        sessions,
        b,
    );
    let worker = panel("WORKER", cells[2], b, CYAN);
    para(
        vec![
            Line::from(bold("● Ready", LIME)),
            plain("Control connected"),
            muted("PID 4821"),
        ],
        worker,
        b,
    );
    para(
        vec![muted(
            "Sessions count connections. Worker readiness does not verify provider delivery.",
        )],
        Rect::new(0, 6, w, if wide { 1 } else { 2 }),
        b,
    );
    let (setup_rect, finding_rect, next) = if wide {
        let c = cols(
            Rect::new(0, 8, w, 9),
            [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)],
        );
        (c[0], c[1], 18)
    } else {
        (Rect::new(0, 9, w, 9), Rect::new(0, 19, w, 9), 29)
    };
    let setup = panel("DOCTOR — SETUP CHECKS", setup_rect, b, CYAN);
    para(
        vec![
            Line::from(bold("✓ PASS", LIME)),
            plain("Config · integration · plugins"),
            muted("– SKIP network probes (offline)"),
            Line::from(""),
            muted(format!("Checked {} UTC · account dev", app.setup_time())),
            muted(format!(
                "Check {} · independent of runtime refresh",
                app.setup_runs
            )),
            Line::from(vec![
                bold("s:recheck setup", CYAN),
                span(" · runs once at startup", SOFT),
            ]),
        ],
        setup,
        b,
    );
    let finding = panel("DIAGNOSTICS — RUNTIME FINDINGS", finding_rect, b, AMBER);
    para(
        vec![Line::from(vec![
            bold("● Recording enabled", LIME),
            span(" · target setting", SOFT),
        ])],
        at(finding, 0, 1),
        b,
    );
    Paragraph::new(Line::from(bold(" ! EARLIER REQUEST TIMEOUT ", AMBER)))
        .style(Style::default().bg(AMBER_BG))
        .render(at(finding, 1, 1), b);
    para(
        vec![
            plain("1 failure · 14:21 UTC · recovery unknown"),
            Line::from(""),
            muted(format!("Snapshot {} UTC", app.time())),
            muted(format!(
                "Cache {:.1}s old · collection OK",
                (3 + app.snapshot_offset).rem_euclid(5) as f64 + 0.2
            )),
            muted("Recorded evidence; current impact unknown."),
        ],
        at(finding, 2, 5),
        b,
    );
    para(
        vec![
            muted(
                "Doctor checks local setup. Diagnostics reads this runtime's available evidence.",
            ),
            muted("Recording is configured on the target; opening the viewer does not enable it."),
        ],
        Rect::new(0, next, w, 2),
        b,
    );
}

fn observability(b: &mut Buffer, app: &App, wide: bool) {
    let w = b.area.width;
    let hero = panel(" WORK NOW ", Rect::new(0, 0, w, 6), b, CYAN);
    let tool_elapsed = app.tool_elapsed();
    let (kind, name, active, elapsed) = match app.sample {
        0 => ("LLM", "claude-sonnet-4-20250514", 1, "4.2s elapsed"),
        1 => ("—", "No open calls observed", 0, ""),
        _ => ("TOOL", "read_file", 1, tool_elapsed.as_str()),
    };
    Paragraph::new(Line::from(bold(
        format!(
            " {active} OBSERVED ACTIVE CALL{} ",
            if active == 1 { "" } else { "S" }
        ),
        BG,
    )))
    .style(Style::default().bg(CYAN))
    .render(at(hero, 0, 1), b);
    para(
        vec![
            Line::from(vec![
                bold(format!("{kind:<7}"), LIME),
                bold(name, FG),
                span(format!("  {elapsed}"), SOFT),
            ]),
            muted("coding-session · start/end events observed"),
        ],
        at(hero, 2, 2),
        b,
    );
    let total_calls = app.llm_calls();
    let reports = total_calls - 1;
    let priced = total_calls - 2;
    let (call_rect, token_rect, lower_y) = if wide {
        let c = cols(
            Rect::new(0, 7, w, 15),
            [Constraint::Percentage(42), Constraint::Percentage(58)],
        );
        (c[0], c[1], 25)
    } else {
        (Rect::new(0, 7, w, 15), Rect::new(0, 23, w, 15), 40)
    };
    let calls = panel(" COMPLETED CALLS ", call_rect, b, CYAN);
    para(vec![muted("calls · linear scale 0–50")], at(calls, 0, 1), b);
    equalizer(
        at(calls, 1, 12),
        b,
        &[
            (
                "LLM",
                total_calls as f64,
                if app.sample == 0 { "18" } else { "19" },
                CYAN,
            ),
            ("Tools", 42.0, "42", LIME),
        ],
        50.0,
    );
    let token = panel(" REPORTED TOKENS ", token_rect, b, CYAN);
    para(
        vec![muted(format!(
            "tokens · 0–10k · {reports}/{total_calls} calls reported"
        ))],
        at(token, 0, 1),
        b,
    );
    let uncached = if app.sample == 0 { 4600.0 } else { 7800.0 };
    let output = if app.sample == 0 { 4200.0 } else { 4360.0 };
    equalizer(
        at(token, 1, 12),
        b,
        &[
            (
                "Uncached input",
                uncached,
                if app.sample == 0 { "4.6k" } else { "7.8k" },
                CYAN,
            ),
            ("Cache read", 8200.0, "8.2k", VIOLET),
            (
                "Output",
                output,
                if app.sample == 0 { "4.2k" } else { "4.36k" },
                LIME,
            ),
        ],
        10000.0,
    );
    let context_y = if wide { 23 } else { 39 };
    para(
        vec![muted(
            "Since 14:14 UTC · 5 calls / row or 1k tokens / row · linear display scales",
        )],
        Rect::new(0, context_y, w, 1),
        b,
    );
    let modules = if wide {
        cols(
            Rect::new(0, lower_y, w, 8),
            [
                Constraint::Percentage(34),
                Constraint::Percentage(34),
                Constraint::Percentage(32),
            ],
        )
    } else {
        std::rc::Rc::from(vec![
            Rect::new(0, lower_y, w, 8),
            Rect::new(0, 49, w, 8),
            Rect::new(0, 58, w, 8),
        ])
    };
    let tools = panel(" TOOLS / COMPLETED CALLS ", modules[0], b, SOFT);
    vu(
        at(tools, 0, 5),
        b,
        &[
            ("read_file", 21.0, "21", LIME),
            ("bash", 12.0, "12", LIME),
            ("apply_patch", 9.0, "9", LIME),
        ],
        30.0,
        "30",
        "15",
        "calls",
    );
    para(vec![muted("Observed executions")], at(tools, 5, 1), b);
    let latency = panel(" LAST COMPLETED LLM / LATENCY ", modules[1], b, SOFT);
    vu(
        at(latency, 0, 5),
        b,
        &[
            (
                "Full call",
                if app.sample == 0 { 2.4 } else { 6.1 },
                if app.sample == 0 { "2.40s" } else { "6.10s" },
                CYAN,
            ),
            (
                "First chunk",
                if app.sample == 0 { 0.38 } else { 0.42 },
                if app.sample == 0 { "0.38s" } else { "0.42s" },
                VIOLET,
            ),
        ],
        10.0,
        "10",
        "5",
        "s",
    );
    para(
        vec![muted("Same call · linear 0–10s scale")],
        at(latency, 5, 1),
        b,
    );
    let cost = panel(" COST / COVERAGE ", modules[2], b, SOFT);
    para(
        vec![
            Line::from(bold(
                if app.sample == 0 {
                    "USD 0.084 · ESTIMATED"
                } else {
                    "USD 0.096 · ESTIMATED"
                },
                FG,
            )),
            muted("Model pricing · partial"),
        ],
        at(cost, 0, 2),
        b,
    );
    gauge(
        at(cost, 2, 1),
        b,
        &format!("usage {reports}/{total_calls}"),
        reports as f64 / total_calls as f64,
        CYAN,
    );
    gauge(
        at(cost, 4, 1),
        b,
        &format!("priced {priced}/{total_calls}"),
        priced as f64 / total_calls as f64,
        VIOLET,
    );
    para(
        vec![muted("Missing values stay unknown")],
        at(cost, 5, 1),
        b,
    );
    let delivery_y = if wide { 34 } else { 67 };
    Paragraph::new(Line::from(vec![
        bold(" ! DELIVERY ", AMBER),
        span(
            "1 earlier export failure · recovery / collector acceptance unverified",
            SOFT,
        ),
    ]))
    .style(Style::default().bg(AMBER_BG))
    .render(Rect::new(0, delivery_y, w, 1), b);
    para(
        vec![muted(
            "Token columns partition reported usage. AV-style LEDs show quantity, not a health score.",
        )],
        Rect::new(0, delivery_y + 1, w, 1),
        b,
    );
}

fn hardware(b: &mut Buffer, app: &App, wide: bool) {
    let w = b.area.width;
    let hero = panel(
        " PROCESS TREE / PID 4821 / LINUX ",
        Rect::new(0, 0, w, 10),
        b,
        CYAN,
    );
    Paragraph::new(Line::from(bold(
        " PARTIAL COVERAGE · 7/8 processes sampled · per-field coverage can differ ",
        AMBER,
    )))
    .style(Style::default().bg(AMBER_BG))
    .render(at(hero, 0, 1), b);
    let c = cols(
        at(hero, 2, 5),
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    );
    para(
        vec![
            Line::from(bold("CPU CONSUMPTION", FG)),
            muted("1.4 / 4.0 logical CPUs"),
        ],
        at(c[0], 0, 2),
        b,
    );
    gauge(at(c[0], 3, 1), b, "35.0% of known capacity", 0.35, CYAN);
    para(
        vec![
            Line::from(bold("ENVIRONMENT-ACCOUNTED MEMORY", FG)),
            muted("1.1 / 8.0 GiB environment limit"),
        ],
        at(c[1], 0, 2),
        b,
    );
    gauge(at(c[1], 3, 1), b, "13.8% of known limit", 1.1 / 8.0, LIME);
    para(
        vec![muted(
            "Owned Linux cgroup · 620 MiB resident · 7 processes · 42 threads · 138 FDs · 0 OOM",
        )],
        at(hero, 7, 1),
        b,
    );
    let (mem_rect, network_rect, y) = if wide {
        let c = cols(
            Rect::new(0, 11, w, 8),
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        );
        (c[0], c[1], 20)
    } else {
        (Rect::new(0, 11, w, 8), Rect::new(0, 20, w, 8), 29)
    };
    let mem = panel(" HOST / SYSTEM MEMORY ", mem_rect, b, SOFT);
    para(
        vec![plain("18.0 / 64.0 GiB used"), muted("44.0 GiB available")],
        at(mem, 0, 2),
        b,
    );
    gauge(
        at(mem, 3, 1),
        b,
        "28.1% system memory used",
        18.0 / 64.0,
        CYAN,
    );
    para(
        vec![muted("System-wide, not attributed to this worker")],
        at(mem, 5, 1),
        b,
    );
    let net = panel(" HOST / NETWORK ", network_rect, b, SOFT);
    vu(
        at(net, 0, 5),
        b,
        &[
            ("Receive", 2.1, "2.1 MiB/s", CYAN),
            ("Transmit", 80.0 / 1024.0, "80 KiB/s", VIOLET),
        ],
        4.0,
        "4",
        "2",
        "MiB/s",
    );
    para(
        vec![muted("System interfaces · RX/TX errors 0/0")],
        at(net, 5, 1),
        b,
    );
    let modules = if wide {
        cols(
            Rect::new(0, y, w, 8),
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
    } else {
        std::rc::Rc::from(vec![Rect::new(0, y, w, 8), Rect::new(0, 38, w, 8)])
    };
    let disk = panel(" PROCESS TREE / DISK I/O ", modules[0], b, SOFT);
    vu(
        at(disk, 0, 5),
        b,
        &[
            ("Read", 1.2, "1.2 MiB/s", CYAN),
            ("Write", 320.0 / 1024.0, "320 KiB/s", VIOLET),
        ],
        2.0,
        "2",
        "1",
        "MiB/s",
    );
    para(
        vec![muted("Rates need consecutive samples")],
        at(disk, 5, 1),
        b,
    );
    let fs = panel(" FILESYSTEM / WORKSPACE ", modules[1], b, SOFT);
    para(
        vec![plain("/workspace"), muted("120 / 512 GiB available")],
        at(fs, 0, 2),
        b,
    );
    gauge(at(fs, 3, 1), b, "23.4% available", 120.0 / 512.0, CYAN);
    para(
        vec![muted("Configured filesystem capacity")],
        at(fs, 5, 1),
        b,
    );
    let note_y = if wide { 29 } else { 47 };
    // The fixture source samples every 5s independently of viewer reads.
    let source_age = (2 + app.snapshot_offset) % 5;
    let source_time = App::clock(86_400 + app.snapshot_offset - source_age);
    para(
        vec![
            muted(format!(
                "Resource sample {source_time} · age {source_age}s · source cadence 5s · GPU not reported"
            )),
            muted(
                "Host/network is system-wide. Meter scales are display ranges, not device limits.",
            ),
        ],
        Rect::new(0, note_y, w, 2),
        b,
    );
}
