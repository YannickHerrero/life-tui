use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, io};

use anyhow::{Context, Result, anyhow};
use rand::Rng;

use crate::grid::{Grid, StepStats};
use crate::patterns::Pattern;
use crate::phase::Phase;
use crate::rle;

pub const MIN_SPEED: u32 = 1;
pub const MAX_SPEED: u32 = 60;
pub const DEFAULT_SPEED: u32 = 10;
pub const HISTORY_LEN: usize = 240;
pub const TOAST_TTL: Duration = Duration::from_millis(2200);

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
    pub paint_state: Option<bool>,
    pub toast: Option<(String, Instant)>,
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
            paint_state: None,
            toast: None,
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

    pub fn clear_grid(&mut self) {
        self.grid.clear();
    }

    /// Stamp a pattern centered on the cursor (with toroidal wrap).
    pub fn stamp_pattern(&mut self, pattern: &Pattern, name: &str) {
        let w = self.grid.width() as i32;
        let h = self.grid.height() as i32;
        if w == 0 || h == 0 {
            return;
        }
        let ox = self.cursor_x as i32 - pattern.width / 2;
        let oy = self.cursor_y as i32 - pattern.height / 2;
        for &(dx, dy) in pattern.cells {
            let x = (ox + dx).rem_euclid(w) as usize;
            let y = (oy + dy).rem_euclid(h) as usize;
            self.grid.set(x, y, true);
        }
        self.set_toast(format!("stamped {name}"));
    }

    pub fn random_fill(&mut self, density: f32) {
        let mut rng = rand::thread_rng();
        let w = self.grid.width();
        let h = self.grid.height();
        for y in 0..h {
            for x in 0..w {
                self.grid.set(x, y, rng.r#gen::<f32>() < density);
            }
        }
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

    pub fn current_toast(&self) -> Option<&str> {
        match &self.toast {
            Some((msg, t)) if t.elapsed() < TOAST_TTL => Some(msg.as_str()),
            _ => None,
        }
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub fn save_pattern(&mut self) {
        match self.try_save() {
            Ok(path) => self.set_toast(format!(
                "saved → {}",
                path.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unknown)")
            )),
            Err(err) => self.set_toast(format!("save failed: {err}")),
        }
    }

    pub fn load_latest_pattern(&mut self) {
        match self.try_load_latest() {
            Ok(name) => self.set_toast(format!("loaded ← {name}")),
            Err(err) => self.set_toast(format!("load failed: {err}")),
        }
    }

    fn try_save(&self) -> Result<PathBuf> {
        let dir = saves_dir()?;
        fs::create_dir_all(&dir)
            .with_context(|| format!("create saves dir {}", dir.display()))?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock before epoch")?
            .as_secs();
        let path = dir.join(format!("{ts}.rle"));
        let body = rle::encode_grid(&self.grid);
        fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
        Ok(path)
    }

    fn try_load_latest(&mut self) -> Result<String> {
        let dir = saves_dir()?;
        let entry = newest_rle(&dir)?
            .ok_or_else(|| anyhow!("no saved patterns in {}", dir.display()))?;
        let body = fs::read_to_string(&entry)
            .with_context(|| format!("read {}", entry.display()))?;
        let cells = rle::decode(&body)?;
        if cells.is_empty() {
            return Err(anyhow!("pattern is empty"));
        }
        // bounding box of the loaded cells
        let max_x = cells.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let max_y = cells.iter().map(|(_, y)| *y).max().unwrap_or(0);
        let pw = max_x + 1;
        let ph = max_y + 1;
        let w = self.grid.width() as i32;
        let h = self.grid.height() as i32;
        let ox = self.cursor_x as i32 - pw / 2;
        let oy = self.cursor_y as i32 - ph / 2;
        for (cx, cy) in cells {
            let x = (ox + cx).rem_euclid(w) as usize;
            let y = (oy + cy).rem_euclid(h) as usize;
            self.grid.set(x, y, true);
        }
        Ok(entry
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string())
    }

    pub fn time_until_next_tick(&self) -> Duration {
        if self.phase != Phase::Run || self.paused {
            return Duration::from_millis(50);
        }
        let period = self.tick_period();
        let elapsed = self.last_tick.elapsed();
        period.saturating_sub(elapsed)
    }
}

fn saves_dir() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or_else(|| anyhow!("HOME not set"))?;
    Ok(base.join("life-tui").join("saves"))
}

fn newest_rle(dir: &std::path::Path) -> Result<Option<PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry.map_err(io::Error::from)?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rle") {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);
        if newest.as_ref().map(|(t, _)| modified > *t).unwrap_or(true) {
            newest = Some((modified, path));
        }
    }
    Ok(newest.map(|(_, p)| p))
}
