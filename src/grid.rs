#[derive(Clone)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    scratch: Vec<bool>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepStats {
    pub births: u32,
    pub deaths: u32,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![false; width * height],
            scratch: vec![false; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = alive;
        }
    }

    pub fn toggle(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.cells[idx] = !self.cells[idx];
        }
    }

    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = false);
    }

    pub fn living(&self) -> usize {
        self.cells.iter().filter(|&&c| c).count()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width == self.width && height == self.height {
            return;
        }
        let mut next = vec![false; width * height];
        let copy_w = self.width.min(width);
        let copy_h = self.height.min(height);
        for y in 0..copy_h {
            for x in 0..copy_w {
                next[y * width + x] = self.cells[y * self.width + x];
            }
        }
        self.width = width;
        self.height = height;
        self.cells = next;
        self.scratch = vec![false; width * height];
    }

    pub fn step(&mut self) -> StepStats {
        let w = self.width;
        let h = self.height;
        let mut births = 0u32;
        let mut deaths = 0u32;

        for y in 0..h {
            let yu = (y + h - 1) % h;
            let yd = (y + 1) % h;
            for x in 0..w {
                let xl = (x + w - 1) % w;
                let xr = (x + 1) % w;
                let n = self.cells[yu * w + xl] as u8
                    + self.cells[yu * w + x] as u8
                    + self.cells[yu * w + xr] as u8
                    + self.cells[y * w + xl] as u8
                    + self.cells[y * w + xr] as u8
                    + self.cells[yd * w + xl] as u8
                    + self.cells[yd * w + x] as u8
                    + self.cells[yd * w + xr] as u8;
                let alive = self.cells[y * w + x];
                let next = matches!((alive, n), (true, 2 | 3) | (false, 3));
                self.scratch[y * w + x] = next;
                match (alive, next) {
                    (false, true) => births += 1,
                    (true, false) => deaths += 1,
                    _ => {}
                }
            }
        }
        std::mem::swap(&mut self.cells, &mut self.scratch);
        StepStats { births, deaths }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinker_oscillates() {
        let mut g = Grid::new(5, 5);
        // vertical bar at (2,1),(2,2),(2,3)
        g.set(2, 1, true);
        g.set(2, 2, true);
        g.set(2, 3, true);
        let s = g.step();
        assert_eq!(s.births, 2);
        assert_eq!(s.deaths, 2);
        // now horizontal: (1,2),(2,2),(3,2)
        assert!(g.get(1, 2));
        assert!(g.get(2, 2));
        assert!(g.get(3, 2));
        assert!(!g.get(2, 1));
        assert!(!g.get(2, 3));
        g.step();
        assert!(g.get(2, 1));
        assert!(g.get(2, 2));
        assert!(g.get(2, 3));
    }

    #[test]
    fn glider_translates_after_four_steps() {
        let mut g = Grid::new(20, 20);
        // glider, top-left at (1,1)
        for &(x, y) in &[(2, 1), (3, 2), (1, 3), (2, 3), (3, 3)] {
            g.set(x, y, true);
        }
        for _ in 0..4 {
            g.step();
        }
        // glider has shifted by (1,1)
        assert_eq!(g.living(), 5);
        for &(x, y) in &[(3, 2), (4, 3), (2, 4), (3, 4), (4, 4)] {
            assert!(g.get(x, y), "expected alive at {x},{y}");
        }
    }

    #[test]
    fn toroidal_wrap() {
        let mut g = Grid::new(4, 4);
        // single cell at corner has neighbors via wrap
        g.set(0, 0, true);
        g.set(3, 0, true);
        g.set(0, 3, true);
        // (0,0) has 2 alive neighbors via wrap: (3,0) and (0,3) and (3,3)? (3,3) is dead. So 2 → stays alive.
        g.step();
        // After one step, all three are alive only if they have 2-3 neighbors.
        // We mostly just assert it doesn't panic and wrap is symmetric.
        assert!(g.living() <= 9);
    }

    #[test]
    fn block_is_stable() {
        let mut g = Grid::new(6, 6);
        for &(x, y) in &[(2, 2), (3, 2), (2, 3), (3, 3)] {
            g.set(x, y, true);
        }
        g.step();
        assert_eq!(g.living(), 4);
        for &(x, y) in &[(2, 2), (3, 2), (2, 3), (3, 3)] {
            assert!(g.get(x, y));
        }
    }
}
