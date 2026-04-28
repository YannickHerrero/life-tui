use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders};

mod app;
mod grid;
mod ui;

use app::App;

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let mut seeded = false;

    while !app.should_quit() {
        terminal.draw(|frame| {
            let outer = Block::default()
                .title(" life-tui ")
                .borders(Borders::ALL);
            let inner = outer.inner(frame.area());
            frame.render_widget(outer, frame.area());

            let (gw, gh) = ui::grid_size_for(inner);
            app.ensure_grid_size(gw, gh);

            if !seeded {
                app.seed_demo();
                seeded = true;
            }

            ui::render_grid(frame, inner, &app.grid);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => app.quit(),
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
