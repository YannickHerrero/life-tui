use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Sparkline};

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

pub fn render_grid(
    frame: &mut Frame,
    area: Rect,
    grid: &Grid,
    cursor: Option<(usize, usize)>,
) {
    let buf = frame.buffer_mut();
    draw_grid(buf, area, grid, cursor);
}

fn draw_grid(buf: &mut Buffer, area: Rect, grid: &Grid, cursor: Option<(usize, usize)>) {
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

            let cursor_top = matches!(cursor, Some((cx, cy)) if cx == x && cy == gy_top);
            let cursor_bot = matches!(cursor, Some((cx, cy)) if cx == x && cy == gy_bot);

            // The cursor's half flips its state and the whole terminal cell is
            // REVERSED, so every up/down step moves a visible half-block — the
            // cursor's half always inverts relative to a non-cursor cell.
            let (top_disp, bot_disp, reversed) = if cursor_top {
                (!top, bot, true)
            } else if cursor_bot {
                (top, !bot, true)
            } else {
                (top, bot, false)
            };
            let glyph = match (top_disp, bot_disp) {
                (false, false) => ' ',
                (true, false) => '▀',
                (false, true) => '▄',
                (true, true) => '█',
            };
            let style = if reversed {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
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

    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(inner);

    render_stats_text(frame, split[0], app);
    if app.phase == Phase::Run {
        render_population_sparkline(frame, split[1], app);
    }
}

fn render_stats_text(frame: &mut Frame, area: Rect, app: &App) {
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

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_population_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    if area.height < 3 {
        return;
    }
    let dim = Style::default().add_modifier(Modifier::DIM);
    let label_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    let chart_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height - 1,
    };

    frame.render_widget(
        Paragraph::new("population").style(dim),
        label_area,
    );

    let data: Vec<u64> = app.population_history.iter().copied().collect();
    let sparkline = Sparkline::default()
        .data(&data)
        .style(Style::default());
    frame.render_widget(sparkline, chart_area);
}

pub fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let dim = Style::default().add_modifier(Modifier::DIM);
    let bold = Style::default().add_modifier(Modifier::BOLD);
    if let Some(toast) = app.current_toast() {
        let text = format!(" {toast} ");
        frame.render_widget(
            Paragraph::new(text).style(bold).alignment(Alignment::Center),
            area,
        );
        return;
    }
    let text = match app.phase {
        Phase::Edit => {
            " hjkl move · space toggle · 1-5 stamp · r rand · c clear · w/L save·load · enter run · ? help "
        }
        Phase::Run => match app.paused {
            true => " space resume · s step · +/- speed · e edit · ? help · q quit ",
            false => " space pause · +/- speed · e edit · ? help · q quit ",
        },
    };
    frame.render_widget(
        Paragraph::new(text).style(dim).alignment(Alignment::Center),
        area,
    );
}


const HELP_LINES: &[(&str, &str)] = &[
    ("EDIT", ""),
    ("  arrows / hjkl", "move cursor"),
    ("  space", "toggle cell at cursor"),
    ("  mouse click + drag", "paint cells"),
    ("  1", "stamp glider"),
    ("  2", "stamp blinker"),
    ("  3", "stamp pulsar"),
    ("  4", "stamp Gosper glider gun"),
    ("  5", "stamp LWSS"),
    ("  r", "random fill (25%)"),
    ("  c", "clear grid"),
    ("  w", "save pattern (RLE)"),
    ("  L", "load most recent save"),
    ("  enter", "start running"),
    ("", ""),
    ("RUN", ""),
    ("  space", "pause / resume"),
    ("  s", "step one generation (paused)"),
    ("  + / -", "speed ± 1 gen/s"),
    ("  e", "return to edit"),
    ("", ""),
    ("GLOBAL", ""),
    ("  ?", "toggle this help"),
    ("  q / esc", "quit"),
];

pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(54, 30, area);
    if popup.width < 30 || popup.height < 8 {
        return;
    }
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" help ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let dim = Style::default().add_modifier(Modifier::DIM);
    let bold = Style::default().add_modifier(Modifier::BOLD);

    let lines: Vec<Line> = HELP_LINES
        .iter()
        .map(|(left, right)| {
            if right.is_empty() && !left.is_empty() {
                Line::from(Span::styled(*left, bold))
            } else if left.is_empty() {
                Line::raw("")
            } else {
                let pad = " ".repeat(28usize.saturating_sub(left.len()));
                Line::from(vec![
                    Span::raw(*left),
                    Span::raw(pad),
                    Span::styled(*right, dim),
                ])
            }
        })
        .collect();

    let mut all_lines = lines;
    all_lines.push(Line::raw(""));
    all_lines.push(Line::from(Span::styled(
        "press any key to dismiss",
        dim,
    )));

    frame.render_widget(
        Paragraph::new(all_lines).alignment(Alignment::Left),
        inner,
    );
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}
