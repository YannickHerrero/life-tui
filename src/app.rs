use crate::grid::Grid;
use crate::phase::Phase;

pub struct App {
    pub grid: Grid,
    pub phase: Phase,
    pub cursor_x: usize,
    pub cursor_y: usize,
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(1, 2),
            phase: Phase::Edit,
            cursor_x: 0,
            cursor_y: 0,
            quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn ensure_grid_size(&mut self, width: usize, height: usize) {
        let w = width.max(1);
        let h = height.max(2);
        self.grid.resize(w, h);
        self.cursor_x = self.cursor_x.min(w.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(h.saturating_sub(1));
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let w = self.grid.width() as i32;
        let h = self.grid.height() as i32;
        if w == 0 || h == 0 {
            return;
        }
        self.cursor_x = ((self.cursor_x as i32 + dx).rem_euclid(w)) as usize;
        self.cursor_y = ((self.cursor_y as i32 + dy).rem_euclid(h)) as usize;
    }

    pub fn toggle_at_cursor(&mut self) {
        self.grid.toggle(self.cursor_x, self.cursor_y);
    }

    pub fn start_run(&mut self) {
        self.phase = Phase::Run;
    }

    pub fn back_to_edit(&mut self) {
        self.phase = Phase::Edit;
    }

    pub fn center_cursor(&mut self) {
        self.cursor_x = self.grid.width() / 2;
        self.cursor_y = self.grid.height() / 2;
    }
}
