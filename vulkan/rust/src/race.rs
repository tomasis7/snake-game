//! Port of racemanager.ts (everything except the HUD drawing, see board.rs).

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaceReason {
    Finish,
    FellBehind,
    #[allow(dead_code)] // part of the TS enum; never raised by the game
    NoLives,
    OpponentOut,
}

pub struct RaceManager {
    start_x: f64,
    finish_x: f64,
    head_x: HashMap<u32, f64>,
    elapsed: f64,
    winner: Option<u32>,
    reason: Option<RaceReason>,
}

impl RaceManager {
    pub fn new(player_numbers: &[u32], start_x: f64, finish_x: f64) -> Self {
        let head_x = player_numbers.iter().map(|&p| (p, start_x)).collect();
        RaceManager { start_x, finish_x, head_x, elapsed: 0.0, winner: None, reason: None }
    }

    pub fn tick(&mut self, dt_ms: f64) {
        if self.winner.is_none() {
            self.elapsed += dt_ms;
        }
    }

    pub fn set_head_x(&mut self, pn: u32, x: f64) {
        self.head_x.insert(pn, x);
    }

    pub fn progress(&self, pn: u32) -> f64 {
        let span = self.finish_x - self.start_x;
        if span <= 0.0 {
            return 1.0;
        }
        let raw = (self.head_x.get(&pn).copied().unwrap_or(self.start_x) - self.start_x) / span;
        raw.clamp(0.0, 1.0)
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed
    }

    #[cfg(test)]
    pub fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    pub fn declare_winner(&mut self, pn: u32, reason: RaceReason) {
        if self.winner.is_some() {
            return;
        }
        self.winner = Some(pn);
        self.reason = Some(reason);
    }

    #[cfg(test)]
    pub fn winner(&self) -> Option<u32> {
        self.winner
    }

    #[cfg(test)]
    pub fn win_reason(&self) -> Option<RaceReason> {
        self.reason
    }
}

pub fn format_time(ms: f64) -> String {
    let total = ms / 1000.0;
    let m = (total / 60.0).floor() as i64;
    let s = (total % 60.0).floor() as i64;
    let tenths = ((total * 10.0) % 10.0).floor() as i64;
    format!("{m}:{s:02}.{tenths}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_start_and_finish() {
        let mut rm = RaceManager::new(&[1, 2], 100.0, 1100.0);
        assert_eq!(rm.progress(1), 0.0);
        rm.set_head_x(1, 1100.0);
        assert_eq!(rm.progress(1), 1.0);
    }

    #[test]
    fn progress_clamps() {
        let mut rm = RaceManager::new(&[1], 100.0, 1100.0);
        rm.set_head_x(1, 2000.0);
        assert_eq!(rm.progress(1), 1.0);
        rm.set_head_x(1, 0.0);
        assert_eq!(rm.progress(1), 0.0);
    }

    #[test]
    fn progress_fraction() {
        let mut rm = RaceManager::new(&[1], 100.0, 1100.0);
        rm.set_head_x(1, 600.0);
        assert!((rm.progress(1) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn progress_zero_width_course() {
        let rm = RaceManager::new(&[1], 500.0, 500.0);
        assert_eq!(rm.progress(1), 1.0);
    }

    #[test]
    fn clock_accumulates() {
        let mut rm = RaceManager::new(&[1], 0.0, 100.0);
        rm.tick(16.0);
        rm.tick(16.0);
        assert_eq!(rm.elapsed_ms(), 32.0);
    }

    #[test]
    fn clock_freezes_after_winner() {
        let mut rm = RaceManager::new(&[1, 2], 0.0, 100.0);
        rm.tick(100.0);
        rm.declare_winner(1, RaceReason::Finish);
        rm.tick(100.0);
        assert_eq!(rm.elapsed_ms(), 100.0);
    }

    #[test]
    fn first_winner_kept() {
        let mut rm = RaceManager::new(&[1, 2], 0.0, 100.0);
        rm.declare_winner(2, RaceReason::OpponentOut);
        rm.declare_winner(1, RaceReason::Finish);
        assert_eq!(rm.winner(), Some(2));
        assert_eq!(rm.win_reason(), Some(RaceReason::OpponentOut));
        assert!(rm.is_over());
    }

    #[test]
    fn no_winner_initially() {
        let rm = RaceManager::new(&[1, 2], 0.0, 100.0);
        assert_eq!(rm.winner(), None);
        assert!(!rm.is_over());
    }

    #[test]
    fn format_time_cases() {
        assert_eq!(format_time(0.0), "0:00.0");
        assert_eq!(format_time(65400.0), "1:05.4");
        assert_eq!(format_time(9900.0), "0:09.9");
        assert_eq!(format_time(42300.0), "0:42.3");
    }
}
