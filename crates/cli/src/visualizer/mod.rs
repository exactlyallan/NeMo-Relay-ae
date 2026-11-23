// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Explicitly attached snapshot viewer. Nothing here records runtime activity.

mod demo;
mod model;
mod render;
pub(crate) mod source;
mod status;

use crate::{
    diagnostics::{self, DoctorProbeMode},
    error::CliError,
    server::GatewayOverrides,
};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures_util::StreamExt;
use model::SetupView;
use source::{FrameRead, ReadError, SnapshotRead, SnapshotSource};
use std::{
    io::{self, IsTerminal},
    process::ExitCode,
    sync::mpsc,
    time::{Duration, Instant},
};

const CADENCES: [u64; 4] = [1, 2, 5, 10];

struct App {
    source_label: String,
    demo: bool,
    snapshot: Option<SnapshotRead>,
    runtime: Option<status::RuntimeStatus>,
    status_at: Option<Instant>,
    status_error: Option<&'static str>,
    details: bool,
    local_account: String,
    received_at: Option<Instant>,
    last_error: Option<String>,
    reading: bool,
    automatic: bool,
    cadence: usize,
    scroll: u16,
    component_scroll: usize,
    setup_tab: bool,
    setup: Option<SetupView>,
    setup_error: Option<String>,
    setup_requested: bool,
    setup_running: bool,
    setup_at: Option<Instant>,
}

impl App {
    fn new(source: &SnapshotSource) -> Self {
        Self {
            source_label: source.label(),
            demo: source.demo,
            snapshot: None,
            runtime: None,
            status_at: None,
            status_error: None,
            details: false,
            local_account: model::local_account(),
            received_at: None,
            last_error: None,
            reading: false,
            automatic: true,
            cadence: 1,
            scroll: 0,
            component_scroll: 0,
            setup_tab: false,
            setup: None,
            setup_error: None,
            setup_requested: false,
            setup_running: false,
            setup_at: None,
        }
    }

    fn accept_frame(&mut self, frame: FrameRead) {
        match frame.status {
            Ok(status) => {
                self.runtime = Some(status);
                self.status_at = Some(frame.sampled_at);
                self.status_error = None;
            }
            Err(reason) => {
                // Never present an old request count as current after a failed status read.
                self.runtime = None;
                self.status_at = None;
                self.status_error = Some(reason);
            }
        }
        self.accept(frame.diagnostics);
    }

    fn accept(&mut self, result: Result<SnapshotRead, ReadError>) {
        self.reading = false;
        match result {
            Ok(read) => {
                let replaced = self.snapshot.as_ref().is_some_and(|old| {
                    old.snapshot.source.instance_id != read.snapshot.source.instance_id
                });
                if replaced {
                    self.component_scroll = 0;
                }
                // A complete replacement is intentional: no deltas, merged sources or viewer history.
                self.snapshot = Some(read);
                self.received_at = Some(Instant::now());
                self.last_error = None;
            }
            Err(error) => {
                if error.discard {
                    self.snapshot = None;
                    self.received_at = None;
                }
                self.last_error = Some(if error.retryable {
                    error.message
                } else {
                    format!(
                        "{}; check diagnostics access and configuration",
                        error.message
                    )
                });
            }
        }
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(CADENCES[self.cadence])
    }

    /// Returns (quit, request immediate read). Pausing never cancels an admitted read.
    fn key(&mut self, key: KeyEvent, viewport: u16) -> (bool, bool) {
        if key.kind == KeyEventKind::Release {
            return (false, false);
        }
        match key.code {
            KeyCode::Char('q') => return (true, false),
            KeyCode::Esc if self.setup_tab || self.details => {
                self.setup_tab = false;
                self.details = false;
                self.scroll = 0;
            }
            KeyCode::Esc => return (true, false),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return (true, false);
            }
            KeyCode::Char(' ') => self.automatic = !self.automatic,
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.cadence = (self.cadence + 1).min(CADENCES.len() - 1)
            }
            KeyCode::Char('-') => self.cadence = self.cadence.saturating_sub(1),
            KeyCode::Char('r') => return (false, true),
            KeyCode::Char('s') => {
                self.setup_tab = true;
                self.details = false;
                if !self.setup_running {
                    self.setup_requested = true;
                }
                self.scroll = 0;
            }
            KeyCode::Char('d') | KeyCode::Tab => {
                self.details = !self.details;
                self.setup_tab = false;
                self.scroll = 0;
                self.component_scroll = 0;
            }
            KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageDown => {
                self.scroll = self
                    .scroll
                    .saturating_add(viewport.saturating_sub(6).max(1))
            }
            KeyCode::PageUp => {
                self.scroll = self
                    .scroll
                    .saturating_sub(viewport.saturating_sub(6).max(1))
            }
            KeyCode::Home => {
                self.scroll = 0;
                self.component_scroll = 0;
            }
            KeyCode::End => self.scroll = u16::MAX,
            KeyCode::Char('j') => self.component_scroll = self.component_scroll.saturating_add(1),
            KeyCode::Char('k') => self.component_scroll = self.component_scroll.saturating_sub(1),
            _ => {}
        }
        (false, false)
    }
}

struct RestoreTerminal;
impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

/// A single bounded blocking reader keeps slow TLS reads off the UI/runtime threads.
/// Dropping its request sender stops the worker after its current bounded read. It is
/// deliberately not Tokio spawn_blocking: runtime teardown must not wait for a stalled read.
struct Reader {
    requests: mpsc::SyncSender<()>,
    results: tokio::sync::mpsc::Receiver<FrameRead>,
}

impl Reader {
    fn start(source: SnapshotSource) -> io::Result<Self> {
        let (requests, receiver) = mpsc::sync_channel(1);
        let (sender, results) = tokio::sync::mpsc::channel(1);
        std::thread::Builder::new()
            .name("relay-visualizer-read".into())
            .spawn(move || {
                let mut frame = 0;
                while receiver.recv().is_ok() {
                    let result = source.collect(frame);
                    frame += 1;
                    if sender.blocking_send(result).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self { requests, results })
    }

    fn request(&self, app: &mut App) {
        if !app.reading && self.requests.try_send(()).is_ok() {
            app.reading = true;
        }
    }
}

pub(crate) async fn run(
    source: SnapshotSource,
    overrides: GatewayOverrides,
) -> Result<ExitCode, CliError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(CliError::Config(
            "--viewer requires an interactive terminal; omit --viewer for JSON inspection".into(),
        ));
    }
    let mut app = App::new(&source);
    let mut reader = Reader::start(source)?;
    // try_init installs Ratatui's restoring panic hook; the guard covers all ordinary exits.
    let _restore = RestoreTerminal;
    let mut terminal = ratatui::try_init()?;
    let mut events = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut next_read = Instant::now();
    let mut setup_tasks = tokio::task::JoinSet::new();
    let terminate = termination_signal();
    tokio::pin!(terminate);
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    loop {
        terminal.draw(|frame| render::draw(frame, &mut app))?;
        tokio::select! {
            _ = &mut terminate => break,
            _ = &mut interrupt => break,
            event = events.next() => {
                let Some(event) = event else { break; };
                if let Event::Key(key) = event? {
                    let previous = (app.cadence, app.automatic, app.setup_tab);
                    let (quit, refresh) = app.key(key, terminal.size()?.height);
                    if quit { break; }
                    if refresh && !app.setup_tab { reader.request(&mut app); }
                    if previous != (app.cadence, app.automatic, app.setup_tab) {
                        next_read = Instant::now() + app.interval();
                    }
                }
            }
            result = reader.results.recv() => {
                let Some(result) = result else {
                    return Err(CliError::Launch("Visualizer read worker stopped".into()));
                };
                app.accept_frame(result);
                next_read = Instant::now() + app.interval();
            }
            _ = tick.tick() => {
                if app.automatic && !app.setup_tab && Instant::now() >= next_read {
                    reader.request(&mut app);
                }
                // Only an explicit Check setup action admits a Doctor run. Repeated keys
                // coalesce while running; polling and navigating never launch probes.
                if app.setup_requested && !app.setup_running {
                    app.setup_requested = false;
                    app.setup_error = None;
                    app.setup = None;
                    if app.demo {
                        app.setup_error = Some("Demo mode does not inspect local setup. Attach a live viewer to read Doctor.".into());
                    } else {
                        app.setup_running = true;
                        let overrides = overrides.clone();
                        setup_tasks.spawn(async move {
                            diagnostics::collect_report(None, DoctorProbeMode::Offline, &overrides)
                                .await.map(|report| SetupView::from(&report)).map_err(|_| ())
                        });
                    }
                }
                if let Some(result) = setup_tasks.try_join_next() {
                    app.setup_running = false;
                    match result {
                        Ok(Ok(view)) => { app.setup = Some(view); app.setup_at = Some(Instant::now()); }
                        _ => app.setup_error = Some("Local offline Doctor report unavailable".into()),
                    }
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

async fn termination_signal() {
    #[cfg(unix)]
    {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
            return;
        }
    }
    std::future::pending::<()>().await;
}

#[cfg(test)]
#[path = "../../tests/coverage/visualizer/app_tests.rs"]
mod tests;
