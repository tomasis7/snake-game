mod atlas;
mod audio;
mod board;
mod button;
mod camera;
mod collision;
mod draw;
mod effects;
mod entity;
mod game;
mod input;
mod instance;
mod levels;
mod pathfinding;
mod player;
mod progress;
mod race;
mod renderer;
mod rng;
mod robot;
mod screens;
mod sound;
mod sprites;
mod text;
mod vec2;
mod viewport;

use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use atlas::Atlas;
use audio::Audio;
use board::DT;
use draw::{Color, Painter};
use game::{Game, ScreenKind};
use input::{Key, Tap};
use renderer::Renderer;
use rng::Rng;
use viewport::Letterbox;

const TITLE: &str = "Furious Snake (Rust)";
const WARMUP_SECS: f64 = 2.0;
/// Frames rendered before a screenshot is taken, so the window size can settle.
const SHOT_SETTLE_FRAMES: u32 = 5;
const SCENE_CAPACITY: usize = 16384;

struct Config {
    bench: bool,
    quads: usize,
    seconds: f64,
    level: u32,
    mute: bool,
    screenshot: Option<(String, ScreenKind)>,
    after_ms: f64,
}

fn fract(v: f32) -> f32 {
    v - v.floor()
}

/// Bench stress quads (see SPEC.md), written after the scene.
fn stress_quads(p: &mut Painter, n: usize, t: f32) {
    for i in 0..n {
        let fi = i as f32;
        let x = fract(fi * 0.618_034 + t * 0.10) * 1200.0;
        let y = fract(fi * 0.754_877_7 + t * 0.07 + 0.05 * (t + fi * 0.001).sin()) * 800.0;
        let color: Color = [fract(fi * 0.13), fract(fi * 0.37), fract(fi * 0.71), 0.6];
        p.solid(x as f64, y as f64, 4.8, 4.8, color);
    }
}

struct App {
    cfg: Config,
    // Declared before the window so the Vulkan surface is destroyed first.
    renderer: Option<Renderer>,
    window: Option<Window>,
    atlas: Atlas,
    game: Game,
    audio: Audio,
    // Timing.
    start: Instant,
    last_frame: Instant,
    last_tick: Instant,
    acc: f64,
    title_t: Instant,
    title_frames: u32,
    // Bench.
    measure_start: Option<Instant>,
    frame_times: Vec<f64>,
    // Screenshot.
    shot_frames: u32,
    shot_simulated: bool,
}

impl App {
    fn new(cfg: Config) -> App {
        let silent = cfg.bench || cfg.screenshot.is_some();
        let game = if cfg.bench {
            Game::for_screen(ScreenKind::Race, cfg.level)
        } else if let Some((_, kind)) = &cfg.screenshot {
            Game::for_screen(*kind, cfg.level)
        } else {
            Game::new(progress::default_path(), Rng::from_time())
        };
        let atlas = atlas::build().unwrap_or_else(|e| {
            eprintln!("atlas build failed: {e}");
            std::process::exit(1);
        });
        let audio = Audio::new(cfg.mute || silent);
        let now = Instant::now();
        App {
            cfg,
            window: None,
            renderer: None,
            atlas,
            game,
            audio,
            start: now,
            last_frame: now,
            last_tick: now,
            acc: 0.0,
            title_t: now,
            title_frames: 0,
            measure_start: None,
            frame_times: Vec::new(),
            shot_frames: 0,
            shot_simulated: false,
        }
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) else { return };
        let size = window.inner_size();
        let t = self.start.elapsed().as_secs_f32();
        let now = Instant::now();

        // Simulation: fixed 60 Hz steps; screenshots run deterministically up front.
        let shot_mode = self.cfg.screenshot.is_some();
        if shot_mode {
            if !self.shot_simulated {
                self.shot_simulated = true;
                let steps = (self.cfg.after_ms / DT).round() as usize;
                for _ in 0..steps {
                    self.game.update();
                }
                self.game.sounds.clear();
            }
            self.acc = 0.0;
        } else {
            self.acc += (now.duration_since(self.last_tick).as_secs_f64() * 1000.0).min(250.0);
            self.last_tick = now;
            while self.acc >= DT {
                self.acc -= DT;
                self.game.update();
                self.audio.play(&self.game.sounds);
                self.game.sounds.clear();
            }
        }
        if self.game.quit {
            event_loop.exit();
            return;
        }

        let capture = shot_mode && {
            self.shot_frames += 1;
            self.shot_frames > SHOT_SETTLE_FRAMES
        };
        let quads = if self.cfg.bench { self.cfg.quads } else { 0 };
        let (game, uv, acc) = (&mut self.game, &self.atlas.uv, self.acc);
        let result = renderer.render((size.width, size.height), capture, |buf| {
            let mut p = Painter::new(buf, uv);
            game.draw(&mut p, acc);
            stress_quads(&mut p, quads, t);
            p.n
        });
        let captured = match result {
            Ok(c) => c,
            Err(e) => {
                eprintln!("render error: {e}");
                std::process::exit(1);
            }
        };
        if let Some(c) = captured {
            let path = &self.cfg.screenshot.as_ref().map(|s| s.0.clone()).unwrap_or_default();
            if let Err(e) = write_png(path, c.width, c.height, &c.rgba) {
                eprintln!("screenshot failed: {e}");
                std::process::exit(1);
            }
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
            let fps = (self.title_frames as f64 / el).round() as u32;
            self.title_frames = 0;
            self.title_t = done;
            if let Some(w) = &self.window {
                w.set_title(&format!("{TITLE} | FPS {fps}"));
            }
        }

        if self.cfg.bench {
            if self.measure_start.is_none() && done.duration_since(self.start).as_secs_f64() >= WARMUP_SECS {
                self.measure_start = Some(done);
            } else if let Some(ms) = self.measure_start {
                self.frame_times.push(ft);
                if done.duration_since(ms).as_secs_f64() >= self.cfg.seconds {
                    let pm = self.renderer.as_ref().map(|r| r.present_mode_name()).unwrap_or("?");
                    let gpu = self.renderer.as_ref().map(|r| r.gpu_name.clone()).unwrap_or_default();
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

fn write_png(path: &str, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        if !dir.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(dir);
        }
    }
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())
}

fn map_key(code: KeyCode) -> Option<Key> {
    Some(match code {
        KeyCode::ArrowUp => Key::Up,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyA => Key::A,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyD => Key::D,
        _ => return None,
    })
}

fn map_tap(code: KeyCode) -> Option<Tap> {
    Some(match code {
        KeyCode::Digit1 | KeyCode::Numpad1 => Tap::Num1,
        KeyCode::Digit2 | KeyCode::Numpad2 => Tap::Num2,
        KeyCode::Enter | KeyCode::NumpadEnter => Tap::Enter,
        KeyCode::Escape => Tap::Escape,
        KeyCode::KeyH => Tap::H,
        KeyCode::KeyN => Tap::N,
        KeyCode::KeyR => Tap::R,
        KeyCode::KeyM => Tap::M,
        KeyCode::KeyG => Tap::G,
        _ => return None,
    })
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // Screenshots use a fixed-size window so tiling compositors keep the requested 1200 x 800.
        let attrs = Window::default_attributes()
            .with_title(TITLE)
            .with_inner_size(PhysicalSize::new(1200, 800))
            .with_resizable(self.cfg.screenshot.is_none());
        let window = match event_loop.create_window(attrs) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("window creation failed: {e}");
                event_loop.exit();
                return;
            }
        };
        let cap = SCENE_CAPACITY + if self.cfg.bench { self.cfg.quads } else { 0 };
        match Renderer::new(&window, cap, self.cfg.bench, &self.atlas.rgba) {
            Ok(r) => {
                eprintln!("GPU: {} | present mode: {}", r.gpu_name, r.present_mode_name());
                self.renderer = Some(r);
            }
            Err(e) => {
                eprintln!("renderer init failed: {e}");
                std::process::exit(1);
            }
        }
        self.window = Some(window);
        let now = Instant::now();
        self.start = now;
        self.last_frame = now;
        self.last_tick = now;
        self.title_t = now;
        // Play the start-up sounds (background music) queued by the game.
        self.audio.play(&self.game.sounds);
        self.game.sounds.clear();
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
                let PhysicalKey::Code(code) = event.physical_key else { return };
                let down = event.state == ElementState::Pressed;
                if let Some(k) = map_key(code) {
                    self.game.input.set_key(k, down);
                }
                if down && !event.repeat {
                    if let Some(t) = map_tap(code) {
                        self.game.input.taps.push(t);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(w) = &self.window {
                    let s = w.inner_size();
                    let (x, y) = Letterbox::fit(s.width, s.height).to_canvas(position.x, position.y);
                    self.game.input.mouse_x = x;
                    self.game.input.mouse_y = y;
                }
            }
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.game.input.mouse_down = state == ElementState::Pressed;
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
                self.last_tick = self.last_frame;
            } else {
                event_loop.set_control_flow(ControlFlow::Poll);
                w.request_redraw();
            }
        }
    }
}

fn parse_args() -> Config {
    let mut cfg = Config {
        bench: false,
        quads: 0,
        seconds: 10.0,
        level: 1,
        mute: false,
        screenshot: None,
        after_ms: 1500.0,
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    let mut shot_path: Option<String> = None;
    let mut shot_kind = ScreenKind::Menu;
    while i < args.len() {
        match args[i].as_str() {
            "--bench" => cfg.bench = true,
            "--mute" => cfg.mute = true,
            "--quads" => {
                i += 1;
                cfg.quads = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
            "--seconds" => {
                i += 1;
                cfg.seconds = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(10.0);
            }
            "--level" => {
                i += 1;
                cfg.level = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1u32).clamp(1, levels::LEVEL_COUNT);
            }
            "--after-ms" => {
                i += 1;
                cfg.after_ms = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(1500.0);
            }
            "--screenshot" => {
                i += 1;
                shot_path = args.get(i).cloned();
            }
            "--screen" => {
                i += 1;
                shot_kind = match args.get(i).map(|s| s.as_str()) {
                    Some("menu") => ScreenKind::Menu,
                    Some("howto") => ScreenKind::HowTo,
                    Some("countdown") => ScreenKind::Countdown,
                    Some("race") => ScreenKind::Race,
                    Some("results") => ScreenKind::Results,
                    other => {
                        eprintln!("unknown --screen {other:?}; expected menu|howto|countdown|race|results");
                        std::process::exit(2);
                    }
                };
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
        i += 1;
    }
    if let Some(p) = shot_path {
        cfg.screenshot = Some((p, shot_kind));
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
