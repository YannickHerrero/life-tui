use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::App;
use crate::grid::Grid;
use crate::phase::Phase;

const PANEL_WIDTH: u16 = 30;
const FOOTER_HEIGHT: u16 = 1;

pub struct Layouts {
    pub grid: Rect,
    pub panel: Option<Rect>,
    pub footer: Rect,
}

/// Compute the sub-rects for grid / panel / footer inside `inner`.
pub fn compute_layout(inner: Rect) -> Layouts {
    let body_footer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Length(FOOTER_HEIGHT)])
        .split(inner);

    let body = body_footer[0];
    let footer = body_footer[1];

    if body.width >= 60 {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(PANEL_WIDTH)])
            .split(body);
        Layouts {
            grid: split[0],
            panel: Some(split[1]),
            footer,
        }
    } else {
        Layouts {
            grid: body,
            panel: None,
            footer,
        }
    }
}

/// The grid dimensions (cell coords) needed to fill `area` in half-block mode.
pub fn grid_size_for(area: Rect) -> (usize, usize) {
    (area.width as usize, (area.height as usize) * 2)
}

pub fn render_grid(frame: &mut Frame, area: Rect, grid: &Grid) {
    let buf = frame.buffer_mut();
    draw_grid(buf, area, grid);
}

fn draw_grid(buf: &mut Buffer, area: Rect, grid: &Grid) {
    let style = Style::default();
    let cols = area.width.min(grid.width() as u16);
    let rows = area.height.min((grid.height() / 2) as u16);

    for ty in 0..rows {
        let gy_top = (ty as usize) * 2;
        let gy_bot = gy_top + 1;
        for tx in 0..cols {
            let x = tx as usize;
            let top = grid.get(x, gy_top);
            let bot = if gy_bot < grid.height() {
                grid.get(x, gy_bot)
            } else {
                false
            };
            let glyph = match (top, bot) {
                (false, false) => ' ',
                (true, false) => '▀',
                (false, true) => '▄',
                (true, true) => '█',
            };
            if let Some(cell) = buf.cell_mut((area.x + tx, area.y + ty)) {
                cell.set_char(glyph).set_style(style);
            }
        }
    }
}

pub fn render_panel(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" stats ")
        .title_alignment(Alignment::Left);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let dim = Style::default().add_modifier(Modifier::DIM);
    let bold = Style::default().add_modifier(Modifier::BOLD);

    let phase = match (app.phase, app.paused) {
        (Phase::Edit, _) => "EDIT",
        (Phase::Run, true) => "RUN · PAUSED",
        (Phase::Run, false) => "RUN",
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("phase     ", dim),
        Span::styled(phase, bold),
    ]));
    lines.push(Line::from(vec![
        Span::styled("generation", dim),
        Span::raw("  "),
        Span::styled(app.generation.to_string(), bold),
    ]));
    lines.push(Line::from(vec![
        Span::styled("living    ", dim),
        Span::styled(app.grid.living().to_string(), bold),
    ]));
    lines.push(Line::from(vec![
        Span::styled("speed     ", dim),
        Span::styled(format!("{} gen/s", app.speed), bold),
    ]));
    lines.push(Line::from(vec![
        Span::styled("births Δ  ", dim),
        Span::styled(format!("+{}", app.last_step.births), Style::default()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("deaths Δ  ", dim),
        Span::styled(format!("-{}", app.last_step.deaths), Style::default()),
    ]));

    if app.phase == Phase::Edit {
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("cursor    ", dim),
            Span::styled(format!("{}, {}", app.cursor_x, app.cursor_y), bold),
        ]));
        lines.push(Line::from(vec![
            Span::styled("size      ", dim),
            Span::styled(format!("{}×{}", app.grid.width(), app.grid.height()), bold),
        ]));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let text = match app.phase {
        Phase::Edit => " arrows/hjkl move · space toggle · enter → run · q quit ",
        Phase::Run => match app.paused {
            true => " space resume · s step · +/- speed · e edit · q quit ",
            false => " space pause · +/- speed · e edit · q quit ",
        },
    };
    frame.render_widget(
        Paragraph::new(text).style(dim).alignment(Alignment::Center),
        area,
    );
}

/// Set the terminal cursor for the edit phase, mapped from cell coords to terminal cell.
pub fn place_edit_cursor(frame: &mut Frame, grid_area: Rect, app: &App) {
    if app.phase != Phase::Edit {
        return;
    }
    let cx = grid_area.x + app.cursor_x as u16;
    let cy = grid_area.y + (app.cursor_y / 2) as u16;
    if cx < grid_area.x + grid_area.width && cy < grid_area.y + grid_area.height {
        frame.set_cursor_position(Position { x: cx, y: cy });
    }
}
