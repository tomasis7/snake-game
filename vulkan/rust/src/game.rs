//! Pure game logic: no Vulkan, no windowing.

use std::collections::VecDeque;

pub const GRID: i32 = 20;
pub const START_TICK_MS: f64 = 150.0;
pub const MIN_TICK_MS: f64 = 60.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
    pub fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 1 } else { seed })
    }
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}

pub struct Game {
    /// Head first.
    pub snake: VecDeque<(i32, i32)>,
    pub dir: Dir,
    queue: Vec<Dir>,
    pub food: (i32, i32),
    pub score: u32,
    pub best: u32,
    pub alive: bool,
    grow: u32,
    rng: Rng,
}

impl Game {
    pub fn new(seed: u32) -> Self {
        let mut g = Game {
            snake: VecDeque::new(),
            dir: Dir::Right,
            queue: Vec::new(),
            food: (0, 0),
            score: 0,
            best: 0,
            alive: true,
            grow: 0,
            rng: Rng::new(seed),
        };
        g.reset();
        g
    }

    pub fn reset(&mut self) {
        let c = GRID / 2;
        self.snake.clear();
        self.snake.push_back((c, c));
        self.snake.push_back((c - 1, c));
        self.snake.push_back((c - 2, c));
        self.dir = Dir::Right;
        self.queue.clear();
        self.score = 0;
        self.alive = true;
        self.grow = 0;
        self.place_food();
    }

    pub fn tick_ms(&self) -> f64 {
        (START_TICK_MS - 5.0 * self.score as f64).max(MIN_TICK_MS)
    }

    /// Queue a turn (max 2 pending); rejects reversals and repeats of the last direction.
    pub fn queue_turn(&mut self, d: Dir) {
        if self.queue.len() >= 2 {
            return;
        }
        let last = self.queue.last().copied().unwrap_or(self.dir);
        if d == last || d == last.opposite() {
            return;
        }
        self.queue.push(d);
    }

    pub fn place_food(&mut self) {
        let free: Vec<(i32, i32)> = (0..GRID)
            .flat_map(|y| (0..GRID).map(move |x| (x, y)))
            .filter(|c| !self.snake.contains(c))
            .collect();
        if free.is_empty() {
            self.alive = false;
            return;
        }
        self.food = free[(self.rng.next() as usize) % free.len()];
    }

    pub fn step(&mut self) {
        if !self.alive {
            return;
        }
        if !self.queue.is_empty() {
            self.dir = self.queue.remove(0);
        }
        let (dx, dy) = self.dir.delta();
        let h = self.snake[0];
        let nh = (h.0 + dx, h.1 + dy);
        if nh.0 < 0 || nh.0 >= GRID || nh.1 < 0 || nh.1 >= GRID {
            self.alive = false;
            return;
        }
        let growing = self.grow > 0;
        // The tail cell is vacated this step unless we are growing.
        let body_len = if growing { self.snake.len() } else { self.snake.len() - 1 };
        if self.snake.iter().take(body_len).any(|&c| c == nh) {
            self.alive = false;
            return;
        }
        self.snake.push_front(nh);
        if growing {
            self.grow -= 1;
        } else {
            self.snake.pop_back();
        }
        if nh == self.food {
            self.score += 1;
            self.best = self.best.max(self.score);
            self.grow += 1;
            self.place_food();
        }
    }

    fn is_safe(&self, d: Dir) -> bool {
        let (dx, dy) = d.delta();
        let h = self.snake[0];
        let nh = (h.0 + dx, h.1 + dy);
        if nh.0 < 0 || nh.0 >= GRID || nh.1 < 0 || nh.1 >= GRID {
            return false;
        }
        let body_len = if self.grow > 0 { self.snake.len() } else { self.snake.len() - 1 };
        !self.snake.iter().take(body_len).any(|&c| c == nh)
    }

    /// Greedy autopilot: step toward food, never into a wall/body if a safe move exists.
    pub fn autopilot(&mut self) {
        let h = self.snake[0];
        let mut best: Option<(i32, Dir)> = None;
        for d in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
            if d == self.dir.opposite() || !self.is_safe(d) {
                continue;
            }
            let (dx, dy) = d.delta();
            let dist = (h.0 + dx - self.food.0).abs() + (h.1 + dy - self.food.1).abs();
            if best.map_or(true, |(bd, _)| dist < bd) {
                best = Some((dist, d));
            }
        }
        if let Some((_, d)) = best {
            self.queue.clear();
            if d != self.dir {
                self.queue.push(d);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_length_three_moving_right() {
        let g = Game::new(42);
        assert_eq!(g.snake.len(), 3);
        assert_eq!(g.dir, Dir::Right);
        assert_eq!(g.snake[0], (10, 10));
    }

    #[test]
    fn moves_one_cell() {
        let mut g = Game::new(42);
        g.food = (0, 0);
        g.step();
        assert_eq!(g.snake[0], (11, 10));
        assert_eq!(g.snake.len(), 3);
    }

    #[test]
    fn eating_grows_and_scores() {
        let mut g = Game::new(42);
        g.food = (11, 10);
        g.step();
        assert_eq!(g.score, 1);
        g.food = (0, 0);
        g.step();
        assert_eq!(g.snake.len(), 4);
        g.step();
        assert_eq!(g.snake.len(), 4);
        assert!(g.tick_ms() < START_TICK_MS);
    }

    #[test]
    fn wall_collision() {
        let mut g = Game::new(42);
        g.food = (0, 0);
        for _ in 0..10 {
            g.step();
        }
        assert!(!g.alive);
    }

    #[test]
    fn self_collision() {
        let mut g = Game::new(42);
        g.snake = VecDeque::from(vec![(5, 5), (5, 6), (6, 6), (6, 5), (6, 4), (5, 4)]);
        g.dir = Dir::Up;
        g.food = (0, 0);
        g.queue_turn(Dir::Right);
        g.step(); // head -> (6,5) which is body
        assert!(!g.alive);
    }

    #[test]
    fn reversal_rejected() {
        let mut g = Game::new(42);
        g.queue_turn(Dir::Left);
        assert!(g.queue.is_empty());
        g.queue_turn(Dir::Up);
        g.queue_turn(Dir::Left); // reversal vs queued Up? no: Left is perpendicular
        g.queue_turn(Dir::Down); // queue full
        assert_eq!(g.queue.len(), 2);
        let mut g = Game::new(42);
        g.queue_turn(Dir::Up);
        g.queue_turn(Dir::Down); // reversal against last queued
        assert_eq!(g.queue, vec![Dir::Up]);
    }

    #[test]
    fn food_never_on_snake() {
        let mut g = Game::new(7);
        for _ in 0..500 {
            g.place_food();
            assert!(!g.snake.contains(&g.food));
        }
    }

    #[test]
    fn autopilot_survives_and_eats() {
        let mut g = Game::new(42);
        for _ in 0..2000 {
            g.autopilot();
            g.step();
            if !g.alive {
                g.reset();
            }
        }
        assert!(g.best > 0);
    }
}
