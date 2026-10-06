//! Port of progress.ts: run-wide state plus best-time persistence.
//! Best times live in `${XDG_DATA_HOME:-~/.local/share}/furious-snake-vulkan/best_times.txt`,
//! one `L<n> <ms>` line per level (shared with the C++ port).

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    OnePlayer,
    TwoPlayer,
}

/// Where best times are persisted. The native app uses a file, the web app localStorage.
pub trait BestStore {
    fn load(&self) -> HashMap<u32, f64>;
    fn save(&self, best: &HashMap<u32, f64>);
}

/// `L<n> <ms>` lines in a text file.
pub struct FileStore(pub PathBuf);

impl BestStore for FileStore {
    fn load(&self) -> HashMap<u32, f64> {
        std::fs::read_to_string(&self.0).ok().map(|t| parse_best_times(&t)).unwrap_or_default()
    }

    fn save(&self, best: &HashMap<u32, f64>) {
        if let Some(dir) = self.0.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let mut levels: Vec<_> = best.iter().collect();
        levels.sort_by_key(|(l, _)| **l);
        let text: String = levels.iter().map(|(l, ms)| format!("L{} {}\n", l, **ms as i64)).collect();
        let _ = std::fs::write(&self.0, text);
    }
}

pub struct Progress {
    pub mode: GameMode,
    pub current_level: u32,
    best: HashMap<u32, f64>,
    store: Option<Box<dyn BestStore>>,
}

pub fn default_path() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_DATA_HOME") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".local/share"),
    };
    Some(base.join("furious-snake-vulkan").join("best_times.txt"))
}

/// Parses the file contents; malformed lines are ignored.
pub fn parse_best_times(text: &str) -> HashMap<u32, f64> {
    let mut m = HashMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(l), Some(ms), None) = (parts.next(), parts.next(), parts.next()) else { continue };
        let Some(n) = l.strip_prefix('L').and_then(|n| n.parse::<u32>().ok()) else { continue };
        if let Ok(v) = ms.parse::<f64>() {
            if v.is_finite() {
                m.insert(n, v);
            }
        }
    }
    m
}

impl Progress {
    /// `path = None` keeps everything in memory (bench, screenshots, tests).
    pub fn new(path: Option<PathBuf>) -> Self {
        Self::with_store(path.map(|p| Box::new(FileStore(p)) as Box<dyn BestStore>))
    }

    pub fn with_store(store: Option<Box<dyn BestStore>>) -> Self {
        let best = store.as_ref().map(|s| s.load()).unwrap_or_default();
        Progress { mode: GameMode::OnePlayer, current_level: 1, best, store }
    }

    pub fn start_run(&mut self, mode: GameMode) {
        self.mode = mode;
        self.current_level = 1;
    }

    pub fn is_last_level(&self) -> bool {
        self.current_level >= 3
    }

    pub fn best_time(&self, level: u32) -> Option<f64> {
        self.best.get(&level).copied()
    }

    /// Returns (best_ms, is_new_best).
    pub fn record_best_time(&mut self, level: u32, time_ms: f64) -> (f64, bool) {
        let time_ms = time_ms.round();
        let prev = self.best_time(level);
        let is_new = prev.map_or(true, |p| time_ms < p);
        if is_new {
            self.best.insert(level, time_ms);
            self.save();
        }
        (if is_new { time_ms } else { prev.unwrap_or(time_ms) }, is_new)
    }

    fn save(&self) {
        if let Some(store) = &self.store {
            store.save(&self.best);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_run_resets() {
        let mut p = Progress::new(None);
        p.start_run(GameMode::OnePlayer);
        p.current_level = 2;
        p.start_run(GameMode::TwoPlayer);
        assert_eq!(p.current_level, 1);
        assert_eq!(p.mode, GameMode::TwoPlayer);
    }

    #[test]
    fn last_level() {
        let mut p = Progress::new(None);
        assert!(!p.is_last_level());
        p.current_level = 3;
        assert!(p.is_last_level());
    }

    #[test]
    fn first_time_is_new_best() {
        let mut p = Progress::new(None);
        assert_eq!(p.best_time(1), None);
        assert_eq!(p.record_best_time(1, 42000.0), (42000.0, true));
        assert_eq!(p.best_time(1), Some(42000.0));
    }

    #[test]
    fn keeps_faster_time() {
        let mut p = Progress::new(None);
        p.record_best_time(1, 42000.0);
        assert_eq!(p.record_best_time(1, 50000.0), (42000.0, false));
        assert_eq!(p.record_best_time(1, 30000.0), (30000.0, true));
    }

    #[test]
    fn per_level_independent() {
        let mut p = Progress::new(None);
        p.record_best_time(1, 40000.0);
        p.record_best_time(2, 60000.0);
        assert_eq!(p.best_time(1), Some(40000.0));
        assert_eq!(p.best_time(2), Some(60000.0));
    }

    #[test]
    fn corrupted_is_no_best() {
        let m = parse_best_times("L1 garbage\nnonsense\nL2 1234\nL3 NaN\n");
        assert_eq!(m.get(&1), None);
        assert_eq!(m.get(&2), Some(&1234.0));
        assert_eq!(m.get(&3), None);
        let mut p = Progress::new(None);
        assert_eq!(p.record_best_time(1, 12345.0), (12345.0, true));
    }

    #[test]
    fn persists_to_file() {
        let dir = std::env::temp_dir().join(format!("fsv_test_{}", std::process::id()));
        let path = dir.join("best_times.txt");
        let _ = std::fs::remove_dir_all(&dir);
        {
            let mut p = Progress::new(Some(path.clone()));
            p.record_best_time(2, 55555.0);
        }
        let p = Progress::new(Some(path));
        assert_eq!(p.best_time(2), Some(55555.0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
