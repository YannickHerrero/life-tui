use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::Rect;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, BorderType, Borders};

mod app;
mod grid;
mod patterns;
mod phase;
mod rle;
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
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();
    let mut centered = false;
    let mut grid_area = Rect::default();

    while !app.should_quit() {
        terminal.draw(|frame| {
            let title = match app.phase {
                Phase::Edit => " life-tui · edit ".to_string(),
                Phase::Run => format!(" life-tui · run · gen {} ", app.generation),
            };
            let outer = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);
            let inner = outer.inner(frame.area());
            frame.render_widget(outer, frame.area());

            let layouts = ui::compute_layout(inner);
            grid_area = layouts.grid;

            let (gw, gh) = ui::grid_size_for(layouts.grid);
            app.ensure_grid_size(gw, gh);

            if !centered {
                app.center_cursor();
                centered = true;
            }

            let cursor = if app.phase == Phase::Edit && !app.show_help {
                Some((app.cursor_x, app.cursor_y))
            } else {
                None
            };
            ui::render_grid(frame, layouts.grid, &app.grid, cursor);
            if let Some(panel) = layouts.panel {
                ui::render_panel(frame, panel, &app);
            }
            ui::render_footer(frame, layouts.footer, &app);

            if app.show_help {
                ui::render_help_overlay(frame, frame.area());
            }
        })?;

        let poll_for = app
            .time_until_next_tick()
            .min(Duration::from_millis(50));
        if event::poll(poll_for)? {
            match event::read()? {
                Event::Mouse(m) => handle_mouse(&mut app, grid_area, m),
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key(&mut app, key.code);
                }
                _ => {}
            }
        }

        app.maybe_tick();
    }

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    if app.show_help {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
            _ => app.close_help(),
        }
        return;
    }
    if matches!(code, KeyCode::Char('?')) {
        app.toggle_help();
        return;
    }
    match (app.phase, code) {
                    (_, KeyCode::Esc | KeyCode::Char('q')) => app.quit(),
                    (Phase::Edit, KeyCode::Left | KeyCode::Char('h')) => app.move_cursor(-1, 0),
                    (Phase::Edit, KeyCode::Right | KeyCode::Char('l')) => app.move_cursor(1, 0),
                    (Phase::Edit, KeyCode::Up | KeyCode::Char('k')) => app.move_cursor(0, -1),
                    (Phase::Edit, KeyCode::Down | KeyCode::Char('j')) => app.move_cursor(0, 1),
                    (Phase::Edit, KeyCode::Char(' ')) => app.toggle_at_cursor(),
                    (Phase::Edit, KeyCode::Char('c')) => app.clear_grid(),
                    (Phase::Edit, KeyCode::Char('r')) => app.random_fill(0.25),
                    (Phase::Edit, KeyCode::Char('1')) => {
                        app.stamp_pattern(&patterns::GLIDER, "glider")
                    }
                    (Phase::Edit, KeyCode::Char('2')) => {
                        app.stamp_pattern(&patterns::BLINKER, "blinker")
                    }
                    (Phase::Edit, KeyCode::Char('3')) => {
                        app.stamp_pattern(&patterns::PULSAR, "pulsar")
                    }
                    (Phase::Edit, KeyCode::Char('4')) => {
                        app.stamp_pattern(&patterns::GOSPER_GLIDER_GUN, "Gosper gun")
                    }
                    (Phase::Edit, KeyCode::Char('5')) => {
                        app.stamp_pattern(&patterns::LWSS, "LWSS")
                    }
                    (Phase::Edit, KeyCode::Char('w')) => app.save_pattern(),
                    (Phase::Edit, KeyCode::Char('L')) => app.load_latest_pattern(),
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

fn handle_mouse(app: &mut App, grid_area: Rect, m: MouseEvent) {
    if app.phase != Phase::Edit {
        return;
    }
    let in_grid = m.column >= grid_area.x
        && m.column < grid_area.x + grid_area.width
        && m.row >= grid_area.y
        && m.row < grid_area.y + grid_area.height;

    match m.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if !in_grid {
                return;
            }
            let (gx, gy_top) = mouse_to_grid(grid_area, m.column, m.row);
            let new_state = !app.grid.get(gx, gy_top);
            app.grid.set(gx, gy_top, new_state);
            if gy_top + 1 < app.grid.height() {
                app.grid.set(gx, gy_top + 1, new_state);
            }
            app.cursor_x = gx;
            app.cursor_y = gy_top;
            app.paint_state = Some(new_state);
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if !in_grid {
                return;
            }
            let Some(state) = app.paint_state else {
                return;
            };
            let (gx, gy_top) = mouse_to_grid(grid_area, m.column, m.row);
            app.grid.set(gx, gy_top, state);
            if gy_top + 1 < app.grid.height() {
                app.grid.set(gx, gy_top + 1, state);
            }
            app.cursor_x = gx;
            app.cursor_y = gy_top;
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.paint_state = None;
        }
        _ => {}
    }
}

fn mouse_to_grid(grid_area: Rect, col: u16, row: u16) -> (usize, usize) {
    let x = (col - grid_area.x) as usize;
    let y_term = (row - grid_area.y) as usize;
    (x, y_term * 2)
}

