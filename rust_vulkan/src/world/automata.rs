use anyhow::Result;
use std::time::Instant;

use crate::infrastructure::app::App;

pub trait Automata {
    fn work(&mut self, app: &App) -> Result<()>;
}

pub struct Spinner {
    pub start: Instant,
}

impl Automata for Spinner {
    fn work(&mut self, app: &App) -> Result<()> {
        Ok(())
    }
}
