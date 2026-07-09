mod app;
mod data;
mod pane;
mod render;
#[cfg(test)]
mod tests;

use anyhow::Result;
use app::App;
use clap::Args;
use crossterm::cursor as cursor_cmds;
use crossterm::{
    event::{self, Event as TermEvent, KeyEventKind},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

#[derive(Args)]
pub(in crate::cli) struct TuiArgs {
    /// Session/status refresh interval in seconds.
    #[arg(long, default_value_t = 2)]
    pub(in crate::cli) refresh_secs: u64,
}

pub(in crate::cli) async fn tui(args: TuiArgs) -> Result<()> {
    let mut app = App::new(Duration::from_secs(args.refresh_secs.max(1)));
    app.refresh().await?;

    let _terminal = TuiTerminal::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut next_refresh = Instant::now() + app.refresh_interval;

    loop {
        app.poll_panes();
        terminal.draw(|f| render::render(f, &mut app))?;

        let wait = next_refresh
            .saturating_duration_since(Instant::now())
            .min(Duration::from_millis(50));
        if event::poll(wait)? {
            match event::read()? {
                TermEvent::Key(key) if key.kind != KeyEventKind::Release => {
                    if !app.handle_key(key).await? {
                        break;
                    }
                }
                TermEvent::Paste(text) => app.forward_paste(text)?,
                _ => {}
            }
        }
        if Instant::now() >= next_refresh {
            if let Err(e) = app.refresh().await {
                app.status = format!("refresh failed: {e:#}");
            }
            next_refresh = Instant::now() + app.refresh_interval;
        }
    }
    Ok(())
}

struct TuiTerminal;

impl TuiTerminal {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, cursor_cmds::Hide)?;
        Ok(Self)
    }
}

impl Drop for TuiTerminal {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), cursor_cmds::Show, LeaveAlternateScreen);
    }
}
