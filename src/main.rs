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
use ratatui::layout::Position;
use ratatui::widgets::{Block, Borders};

mod app;
mod grid;
mod phase;
mod ui;

use app::App;
use phase::Phase;

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
    let mut centered = false;

    while !app.should_quit() {
        terminal.draw(|frame| {
            let outer = Block::default()
                .title(format!(" life-tui — {} ", phase_label(app.phase)))
                .borders(Borders::ALL);
            let inner = outer.inner(frame.area());
            frame.render_widget(outer, frame.area());

            let (gw, gh) = ui::grid_size_for(inner);
            app.ensure_grid_size(gw, gh);

            if !centered {
                app.center_cursor();
                centered = true;
            }

            ui::render_grid(frame, inner, &app.grid);

            if app.phase == Phase::Edit {
                let cx = inner.x + app.cursor_x as u16;
                let cy = inner.y + (app.cursor_y / 2) as u16;
                frame.set_cursor_position(Position { x: cx, y: cy });
            }
        })?;

        let poll_for = app
            .time_until_next_tick()
            .min(Duration::from_millis(50));
        if event::poll(poll_for)? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match (app.phase, key.code) {
                    (_, KeyCode::Esc | KeyCode::Char('q')) => app.quit(),
                    (Phase::Edit, KeyCode::Left | KeyCode::Char('h')) => app.move_cursor(-1, 0),
                    (Phase::Edit, KeyCode::Right | KeyCode::Char('l')) => app.move_cursor(1, 0),
                    (Phase::Edit, KeyCode::Up | KeyCode::Char('k')) => app.move_cursor(0, -1),
                    (Phase::Edit, KeyCode::Down | KeyCode::Char('j')) => app.move_cursor(0, 1),
                    (Phase::Edit, KeyCode::Char(' ')) => app.toggle_at_cursor(),
                    (Phase::Edit, KeyCode::Enter) => app.start_run(),
                    (Phase::Run, KeyCode::Char('e')) => app.back_to_edit(),
                    (Phase::Run, KeyCode::Char(' ')) => app.toggle_pause(),
                    (Phase::Run, KeyCode::Char('s')) => {
                        if app.paused {
                            app.step_once();
                        }
                    }
                    (Phase::Run, KeyCode::Char('+') | KeyCode::Char('=')) => app.faster(),
                    (Phase::Run, KeyCode::Char('-') | KeyCode::Char('_')) => app.slower(),
                    _ => {}
                }
            }
        }

        app.maybe_tick();
    }

    Ok(())
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Edit => "edit",
        Phase::Run => "run",
    }
}
