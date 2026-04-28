use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::grid::Grid;

/// Render `grid` into `area` using half-block characters:
/// each terminal row holds two grid rows (top half / bottom half).
pub fn render_grid(frame: &mut Frame, area: Rect, grid: &Grid) {
    let buf = frame.buffer_mut();
    draw_grid(buf, area, grid);
}

pub fn draw_grid(buf: &mut Buffer, area: Rect, grid: &Grid) {
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

/// The grid dimensions (cell coords) needed to fill `area` in half-block mode.
pub fn grid_size_for(area: Rect) -> (usize, usize) {
    (area.width as usize, (area.height as usize) * 2)
}
