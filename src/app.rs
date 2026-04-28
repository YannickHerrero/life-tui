use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::grid::{Grid, StepStats};
use crate::phase::Phase;

pub const MIN_SPEED: u32 = 1;
pub const MAX_SPEED: u32 = 60;
pub const DEFAULT_SPEED: u32 = 10;
pub const HISTORY_LEN: usize = 240;

pub struct App {
    pub grid: Grid,
    pub phase: Phase,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub generation: u64,
    pub last_step: StepStats,
    pub speed: u32,
    pub paused: bool,
    pub population_history: VecDeque<u64>,
    last_tick: Instant,
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(1, 2),
            phase: Phase::Edit,
            cursor_x: 0,
            cursor_y: 0,
            generation: 0,
            last_step: StepStats::default(),
            speed: DEFAULT_SPEED,
            paused: false,
            population_history: VecDeque::with_capacity(HISTORY_LEN),
            last_tick: Instant::now(),
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
        self.generation = 0;
        self.last_step = StepStats::default();
        self.paused = false;
        self.last_tick = Instant::now();
        self.population_history.clear();
        self.record_population();
    }

    fn record_population(&mut self) {
        if self.population_history.len() == HISTORY_LEN {
            self.population_history.pop_front();
        }
        self.population_history.push_back(self.grid.living() as u64);
    }

    pub fn back_to_edit(&mut self) {
        self.phase = Phase::Edit;
    }

    pub fn center_cursor(&mut self) {
        self.cursor_x = self.grid.width() / 2;
        self.cursor_y = self.grid.height() / 2;
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.last_tick = Instant::now();
    }

    pub fn step_once(&mut self) {
        self.last_step = self.grid.step();
        self.generation += 1;
        self.record_population();
    }

    pub fn faster(&mut self) {
        self.speed = (self.speed + 1).min(MAX_SPEED);
    }

    pub fn slower(&mut self) {
        self.speed = self.speed.saturating_sub(1).max(MIN_SPEED);
    }

    fn tick_period(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.speed as f64)
    }

    /// Advance the simulation if running, not paused, and the tick period elapsed.
    /// Catches up by stepping multiple times if we fell behind, capped per call.
    pub fn maybe_tick(&mut self) {
        if self.phase != Phase::Run || self.paused {
            return;
        }
        let period = self.tick_period();
        let mut steps = 0;
        while self.last_tick.elapsed() >= period && steps < 8 {
            self.step_once();
            self.last_tick += period;
            steps += 1;
        }
    }

    /// How long until the next tick should fire (used to size the event poll window).
    pub fn time_until_next_tick(&self) -> Duration {
        if self.phase != Phase::Run || self.paused {
            return Duration::from_millis(50);
        }
        let period = self.tick_period();
        let elapsed = self.last_tick.elapsed();
        period.saturating_sub(elapsed)
    }
}
