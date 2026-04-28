pub struct App {
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { quit: false }
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn quit(&mut self) {
        self.quit = true;
    }
}
