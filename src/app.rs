use crate::grid::Grid;

pub struct App {
    pub grid: Grid,
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(1, 2),
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
        self.grid.resize(width.max(1), height.max(2));
    }

    /// Seed a small demo pattern at the center of the grid (used until edit phase
    /// hotkeys exist). Blinker for now.
    pub fn seed_demo(&mut self) {
        self.grid.clear();
        let cx = self.grid.width() / 2;
        let cy = self.grid.height() / 2;
        for dy in -1i32..=1 {
            self.grid.set(cx, (cy as i32 + dy).max(0) as usize, true);
        }
    }
}
