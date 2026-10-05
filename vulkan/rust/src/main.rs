mod game;
mod renderer;

use game::{Dir, Game, GRID};
use renderer::{Instance, Renderer};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const TITLE: &str = "Vulkan Snake (Rust)";
const WARMUP_SECS: f64 = 2.0;

struct Config {
    bench: bool,
    quads: usize,
    seconds: f64,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Hex sRGB colour converted to linear (the swapchain is an sRGB format).
fn hex(c: u32, a: f32) -> [f32; 4] {
    let ch = |s: u32| srgb_to_linear(((c >> s) & 255) as f32 / 255.0);
    [ch(16), ch(8), ch(0), a]
}

fn fract(v: f32) -> f32 {
    v - v.floor()
}

/// Writes board, snake, food (and stress quads in bench mode). Returns instance count.
fn fill_instances(out: &mut [Instance], g: &Game, quads: usize, t: f32) -> usize {
    let cell = 1.0 / GRID as f32;
    let inset = 0.05 * cell;
    let mut n = 0;
    let board = hex(0x1e1e2a, 1.0);
    for y in 0..GRID {
        for x in 0..GRID {
            out[n] = Instance {
                pos: [x as f32 * cell + inset, y as f32 * cell + inset],
                size: [cell - 2.0 * inset, cell - 2.0 * inset],
                color: board,
            };
            n += 1;
        }
    }
    let len = g.snake.len();
    for (i, &(x, y)) in g.snake.iter().enumerate() {
        let color = if !g.alive {
            hex(0xc0392b, 1.0)
        } else if i == 0 {
            hex(0x7CFC00, 1.0)
        } else {
            let mut c = hex(0x32CD32, 1.0);
            let k = 1.0 - 0.45 * (i as f32 / len as f32);
            c[0] *= k;
            c[1] *= k;
            c[2] *= k;
            c
        };
        out[n] = Instance {
            pos: [x as f32 * cell + inset, y as f32 * cell + inset],
            size: [cell - 2.0 * inset, cell - 2.0 * inset],
            color,
        };
        n += 1;
    }
    out[n] = Instance {
        pos: [g.food.0 as f32 * cell + inset, g.food.1 as f32 * cell + inset],
        size: [cell - 2.0 * inset, cell - 2.0 * inset],
        color: hex(0xFF4757, 1.0),
    };
    n += 1;
    for i in 0..quads {
        let fi = i as f32;
        out[n] = Instance {
            pos: [
                fract(fi * 0.6180339 + t * 0.10),
                fract(fi * 0.3819660 + t * 0.07 + 0.05 * (t + fi * 0.001).sin()),
            ],
            size: [0.004, 0.004],
            color: [fract(fi * 0.13), fract(fi * 0.37), fract(fi * 0.71), 0.6],
        };
        n += 1;
    }
    n
}

struct App {
    cfg: Config,
    // Renderer is declared before window so it drops first.
    renderer: Option<Renderer>,
    window: Option<Window>,
    game: Game,
    paused: bool,
    acc: f64,
    last_frame: Instant,
    start: Instant,
    // title / fps
    title_t: Instant,
    title_frames: u32,
    fps: u32,
    // bench
    frame_times: Vec<f64>,
    measure_start: Option<Instant>,
}

impl App {
    fn new(cfg: Config) -> Self {
        let now = Instant::now();
        App {
            game: Game::new(if cfg.bench { 42 } else { (now.elapsed().subsec_nanos() ^ 0x9E3779B9) | 1 }),
            cfg,
            renderer: None,
            window: None,
            paused: false,
            acc: 0.0,
            last_frame: now,
            start: now,
            title_t: now,
            title_frames: 0,
            fps: 0,
            frame_times: Vec::new(),
            measure_start: None,
        }
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) else {
            return;
        };
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f64().min(0.25);

        // Fixed-timestep logic.
        if !self.paused {
            self.acc += dt * 1000.0;
            loop {
                let tick = self.game.tick_ms();
                if self.acc < tick {
                    break;
                }
                self.acc -= tick;
                if self.cfg.bench {
                    self.game.autopilot();
                    self.game.step();
                    if !self.game.alive {
                        self.game.reset();
                    }
                } else {
                    self.game.step();
                }
            }
        }

        let size = window.inner_size();
        let t = self.start.elapsed().as_secs_f32();
        let game = &self.game;
        let quads = if self.cfg.bench { self.cfg.quads } else { 0 };
        if let Err(e) = renderer.render((size.width, size.height), |buf| fill_instances(buf, game, quads, t)) {
            eprintln!("render error: {e}");
            event_loop.exit();
            return;
        }

        let done = Instant::now();
        let ft = done.duration_since(self.last_frame).as_secs_f64() * 1000.0;
        self.last_frame = done;

        // Title refresh ~2x per second.
        self.title_frames += 1;
        let el = done.duration_since(self.title_t).as_secs_f64();
        if el >= 0.5 {
            self.fps = (self.title_frames as f64 / el).round() as u32;
            self.title_frames = 0;
            self.title_t = done;
            window.set_title(&format!(
                "{TITLE} | Score {} | Best {} | FPS {}",
                self.game.score, self.game.best, self.fps
            ));
        }

        if self.cfg.bench {
            if self.measure_start.is_none() && done.duration_since(self.start).as_secs_f64() >= WARMUP_SECS {
                self.measure_start = Some(done);
            } else if let Some(ms) = self.measure_start {
                self.frame_times.push(ft);
                if done.duration_since(ms).as_secs_f64() >= self.cfg.seconds {
                    let pm = renderer.present_mode_name();
                    let gpu = renderer.gpu_name.clone();
                    self.report(pm, &gpu);
                    event_loop.exit();
                }
            }
        }
    }

    fn report(&self, present_mode: &str, gpu: &str) {
        let mut v = self.frame_times.clone();
        let n = v.len();
        if n == 0 {
            eprintln!("no frames measured");
            return;
        }
        let sum: f64 = v.iter().sum();
        let avg_ms = sum / n as f64;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p99_idx = (((0.99 * n as f64).ceil() as usize).max(1) - 1).min(n - 1);
        let worst = ((n as f64 * 0.01).floor() as usize).max(1);
        let worst_mean = v[n - worst..].iter().sum::<f64>() / worst as f64;
        println!(
            "{{\"impl\":\"rust\",\"present_mode\":\"{}\",\"quads\":{},\"seconds\":{},\"frames\":{},\"avg_fps\":{:.2},\"p1_low_fps\":{:.2},\"avg_ms\":{:.2},\"p99_ms\":{:.2},\"gpu\":\"{}\"}}",
            present_mode,
            self.cfg.quads,
            self.cfg.seconds,
            n,
            1000.0 / avg_ms,
            1000.0 / worst_mean,
            avg_ms,
            v[p99_idx],
            gpu.replace('\\', "\\\\").replace('"', "\\\"")
        );
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(PhysicalSize::new(800, 800));
        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("window creation failed: {e}");
                event_loop.exit();
                return;
            }
        };
        let cap = 1024 + if self.cfg.bench { self.cfg.quads } else { 0 };
        match Renderer::new(&window, cap, self.cfg.bench) {
            Ok(r) => {
                eprintln!("GPU: {} | present mode: {}", r.gpu_name, r.present_mode_name());
                self.renderer = Some(r);
            }
            Err(e) => {
                eprintln!("renderer init failed: {e}");
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
        let now = Instant::now();
        self.start = now;
        self.last_frame = now;
        self.title_t = now;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(r) = self.renderer.as_mut() {
                    r.mark_resized();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed || event.repeat {
                    return;
                }
                let PhysicalKey::Code(code) = event.physical_key else { return };
                match code {
                    KeyCode::ArrowUp | KeyCode::KeyW => self.game.queue_turn(Dir::Up),
                    KeyCode::ArrowDown | KeyCode::KeyS => self.game.queue_turn(Dir::Down),
                    KeyCode::ArrowLeft | KeyCode::KeyA => self.game.queue_turn(Dir::Left),
                    KeyCode::ArrowRight | KeyCode::KeyD => self.game.queue_turn(Dir::Right),
                    KeyCode::KeyR | KeyCode::Enter | KeyCode::NumpadEnter => {
                        self.game.reset();
                        self.paused = false;
                        self.acc = 0.0;
                    }
                    KeyCode::KeyP | KeyCode::Space => self.paused = !self.paused,
                    KeyCode::Escape => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            let s = w.inner_size();
            if s.width == 0 || s.height == 0 {
                // Minimised: sleep until the next event.
                event_loop.set_control_flow(ControlFlow::Wait);
                self.last_frame = Instant::now();
            } else {
                event_loop.set_control_flow(ControlFlow::Poll);
                w.request_redraw();
            }
        }
    }
}

fn parse_args() -> Config {
    let mut cfg = Config { bench: false, quads: 0, seconds: 10.0 };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--bench" => cfg.bench = true,
            "--quads" => {
                i += 1;
                cfg.quads = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--seconds" => {
                i += 1;
                cfg.seconds = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(10.0);
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
        i += 1;
    }
    cfg
}

fn main() {
    let cfg = parse_args();
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(cfg);
    event_loop.run_app(&mut app).expect("event loop run");
}
