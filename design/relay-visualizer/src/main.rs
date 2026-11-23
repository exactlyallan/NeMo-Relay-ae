// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Throwaway design fixture. No Relay connection, probes, payloads, or live collection.
mod views;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    style::{Color, Modifier},
};
use std::{
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

// The startup Doctor result is a fixture, collected once independently of the
// synthetic runtime reads. No key or timer invokes a real command or daemon.
struct App {
    page: usize,
    sample: usize,
    scroll: u16,
    details: bool,
    rate_index: usize,
    revision: u64,
    snapshot_offset: u64,
    setup_offset: u64,
    setup_runs: u64,
    tool_started_offset: Option<u64>,
    started: Instant,
}
impl Default for App {
    fn default() -> Self {
        Self {
            page: 0,
            sample: 0,
            scroll: 0,
            details: false,
            rate_index: 1,
            revision: 0,
            snapshot_offset: 0,
            setup_offset: 0,
            setup_runs: 1,
            tool_started_offset: None,
            started: Instant::now(),
        }
    }
}
impl App {
    fn rate_seconds(&self) -> Option<u64> {
        [Some(1), Some(5), Some(20), None][self.rate_index]
    }
    fn rate_label(&self) -> &'static str {
        ["1s", "5s", "20s", "paused"][self.rate_index]
    }
    fn refresh(&mut self) {
        // Monotonic fixture time; rapid manual reads advance the sample story
        // without wrapping its counters or rerunning the startup setup check.
        self.snapshot_offset = (self.snapshot_offset + 2).max(self.started.elapsed().as_secs());
        self.revision += 1;
        self.sample = (self.sample + 1).min(2);
        if self.sample == 2 && self.tool_started_offset.is_none() {
            self.tool_started_offset = Some(self.snapshot_offset);
        }
    }
    fn key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('1'..='3') => {
                if let KeyCode::Char(c) = code {
                    self.page = c as usize - '1' as usize;
                }
                self.scroll = 0;
            }
            KeyCode::Tab | KeyCode::Right => {
                self.page = (self.page + 1) % 3;
                self.scroll = 0;
            }
            KeyCode::BackTab | KeyCode::Left => {
                self.page = (self.page + 2) % 3;
                self.scroll = 0;
            }
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('s') => {
                self.setup_offset = self.snapshot_offset.max(self.started.elapsed().as_secs());
                self.setup_runs += 1;
                self.page = 0;
                self.scroll = 0;
            }
            KeyCode::Char('d') => self.details = !self.details,
            KeyCode::Char(' ') => self.rate_index = (self.rate_index + 1) % 4,
            KeyCode::Down | KeyCode::Char('j') => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(12),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(12),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = u16::MAX,
            _ => {}
        }
        false
    }
    fn requests(&self) -> u64 {
        if self.sample == 0 { 1 } else { 0 }
    }
    fn llm_calls(&self) -> u64 {
        if self.sample == 0 { 18 } else { 19 }
    }
    fn clock(offset: u64) -> String {
        let seconds = (14 * 3600 + 32 * 60 + 50 + offset) % (24 * 3600);
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds / 60 % 60,
            seconds % 60
        )
    }
    fn time(&self) -> String {
        Self::clock(self.snapshot_offset)
    }
    fn setup_time(&self) -> String {
        Self::clock(self.setup_offset)
    }
    fn tool_elapsed(&self) -> String {
        format!(
            "{:.1}s elapsed",
            0.7 + self
                .snapshot_offset
                .saturating_sub(self.tool_started_offset.unwrap_or(self.snapshot_offset))
                as f64
        )
    }
}

struct Restore;
impl Drop for Restore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn rgb(color: Color, fallback: [u8; 3]) -> [u8; 3] {
    match color {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::White => [255, 255, 255],
        _ => fallback,
    }
}
/// Export the actual Ratatui cell buffer, never a browser recreation of the layout.
fn export(buffer: &Buffer, destination: &Path) -> io::Result<()> {
    let mut text = String::new();
    let mut cells = Vec::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let c = &buffer[(x, y)];
            text.push_str(c.symbol());
            cells.push(serde_json::json!([
                c.symbol(),
                rgb(c.fg, [219, 232, 239]),
                rgb(c.bg, [10, 17, 23]),
                c.modifier.contains(Modifier::BOLD),
                c.modifier.contains(Modifier::DIM),
                c.modifier.contains(Modifier::REVERSED)
            ]));
        }
        text.push('\n');
    }
    let data = serde_json::json!({"width":buffer.area.width,"height":buffer.area.height,"cells":cells,"renderer":"ratatui 0.30.2 / TestBackend"});
    fs::write(destination.with_extension("txt"), text)?;
    fs::write(
        destination.with_extension("json"),
        serde_json::to_vec(&data)?,
    )
}

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--export") {
        let destination = args
            .get(2)
            .expect("--export requires a destination directory");
        fs::create_dir_all(destination)?;
        for (width, height) in [(120, 44), (120, 32), (100, 40), (80, 32)] {
            for page in 0..3 {
                let mut app = App {
                    page,
                    ..Default::default()
                };
                let mut terminal = Terminal::new(TestBackend::new(width, height))
                    .expect("TestBackend is infallible");
                terminal
                    .draw(|f| views::draw(f, &mut app))
                    .expect("TestBackend is infallible");
                let name = ["overview", "observability", "hardware"][page];
                export(
                    terminal.backend().buffer(),
                    &Path::new(destination).join(format!("{name}-{width}x{height}")),
                )?;
            }
        }
        return Ok(());
    }
    let mut app = App::default();
    let _restore = Restore;
    let mut terminal = ratatui::try_init()?;
    let mut next_read = Some(Instant::now() + Duration::from_secs(5));
    loop {
        terminal.draw(|f| views::draw(f, &mut app))?;
        let previous = (app.rate_index, app.revision);
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release
                    && ((key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL))
                        || app.key(key.code))
                {
                    break;
                }
            }
        }
        if previous != (app.rate_index, app.revision) {
            next_read = app
                .rate_seconds()
                .map(|s| Instant::now() + Duration::from_secs(s));
        }
        if next_read.is_some_and(|deadline| Instant::now() >= deadline) {
            app.refresh();
            next_read = app
                .rate_seconds()
                .map(|s| Instant::now() + Duration::from_secs(s));
        }
    }
    Ok(())
}
