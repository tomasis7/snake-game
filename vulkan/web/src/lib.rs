//! Furious Snake in the browser: furious_core + wgpu (WebGPU, WebGL2 fallback).

mod audio;
mod gpu;

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use furious_core::atlas::{self, Atlas};
use furious_core::bench::{report_json, stress_quads};
use furious_core::board::DT;
use furious_core::draw::Painter;
use furious_core::game::{Game, ScreenKind};
use furious_core::input::{Key, Tap};
use furious_core::instance::Instance;
use furious_core::levels;
use furious_core::progress::{BestStore, Progress};
use furious_core::rng::Rng;
use furious_core::viewport::{CANVAS_H, CANVAS_W};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use audio::Audio;
use gpu::Gpu;

const SCENE_CAPACITY: usize = 16384;
const WARMUP_SECS: f64 = 2.0;

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

fn now_ms() -> f64 {
    window().performance().expect("performance").now()
}

fn show_message(text: &str) {
    let doc = window().document().expect("document");
    if let Some(c) = doc.get_element_by_id("game") {
        let _ = c.set_attribute("style", "display:none");
    }
    if let Some(m) = doc.get_element_by_id("message") {
        let _ = m.set_attribute("style", "display:block");
        m.set_text_content(Some(text));
    }
}

/// Best times in localStorage under the TS game's keys.
struct LocalStore;

fn storage() -> Option<web_sys::Storage> {
    window().local_storage().ok().flatten()
}

fn best_key(level: u32) -> String {
    format!("furious-snake-best-time-L{level}")
}

impl BestStore for LocalStore {
    fn load(&self) -> HashMap<u32, f64> {
        let mut m = HashMap::new();
        if let Some(s) = storage() {
            for level in 1..=levels::LEVEL_COUNT {
                if let Ok(Some(raw)) = s.get_item(&best_key(level)) {
                    if let Ok(v) = raw.parse::<f64>() {
                        if v.is_finite() {
                            m.insert(level, v);
                        }
                    }
                }
            }
        }
        m
    }

    fn save(&self, best: &HashMap<u32, f64>) {
        if let Some(s) = storage() {
            for (level, ms) in best {
                let _ = s.set_item(&best_key(*level), &ms.to_string());
            }
        }
    }
}

struct Config {
    bench: bool,
    quads: usize,
    seconds: f64,
    level: u32,
    mute: bool,
}

fn parse_config() -> Config {
    let search = window().location().search().unwrap_or_default();
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok();
    let get = |k: &str| params.as_ref().and_then(|p| p.get(k));
    Config {
        bench: params.as_ref().is_some_and(|p| p.has("bench")),
        quads: get("quads").and_then(|s| s.parse().ok()).unwrap_or(0),
        seconds: get("seconds").and_then(|s| s.parse().ok()).unwrap_or(10.0),
        level: get("level").and_then(|s| s.parse().ok()).unwrap_or(1u32).clamp(1, levels::LEVEL_COUNT),
        mute: params.as_ref().is_some_and(|p| p.has("mute")),
    }
}

fn map_key(code: &str) -> Option<Key> {
    Some(match code {
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "KeyW" => Key::W,
        "KeyA" => Key::A,
        "KeyS" => Key::S,
        "KeyD" => Key::D,
        _ => return None,
    })
}

fn map_tap(code: &str) -> Option<Tap> {
    Some(match code {
        "Digit1" | "Numpad1" => Tap::Num1,
        "Digit2" | "Numpad2" => Tap::Num2,
        "Enter" | "NumpadEnter" => Tap::Enter,
        "Escape" => Tap::Escape,
        "KeyH" => Tap::H,
        "KeyN" => Tap::N,
        "KeyR" => Tap::R,
        "KeyM" => Tap::M,
        "KeyG" => Tap::G,
        _ => return None,
    })
}

struct App {
    game: Game,
    audio: Audio,
    atlas: Atlas,
    gpu: Gpu,
    scene: Vec<Instance>,
    canvas: web_sys::HtmlCanvasElement,
    last_tick: f64,
    acc: f64,
    title_t: f64,
    title_frames: u32,
}

impl App {
    fn title_prefix(&self) -> &'static str {
        if self.gpu.api == "webgpu" {
            "Furious Snake (WASM, WebGPU)"
        } else {
            "Furious Snake (WASM, WebGL2)"
        }
    }

    fn frame(&mut self, now: f64) {
        self.acc += (now - self.last_tick).min(250.0);
        self.last_tick = now;
        while self.acc >= DT {
            self.acc -= DT;
            self.game.update();
            self.audio.play(&self.game.sounds);
            self.game.sounds.clear();
        }
        // There is nothing to quit to in a browser tab.
        self.game.quit = false;

        let n = {
            let mut p = Painter::new(&mut self.scene, &self.atlas.uv);
            self.game.draw(&mut p, self.acc);
            p.n
        };
        self.gpu.upload(&self.scene, n);
        self.gpu.render_surface(n);

        self.title_frames += 1;
        let el = (now - self.title_t) / 1000.0;
        if el >= 0.5 {
            let fps = (self.title_frames as f64 / el).round() as u32;
            self.title_frames = 0;
            self.title_t = now;
            if let Some(doc) = window().document() {
                doc.set_title(&format!("{} | FPS {fps}", self.title_prefix()));
            }
        }
    }
}

fn add_listener<E: wasm_bindgen::convert::FromWasmAbi + 'static>(target: &web_sys::EventTarget, name: &str, f: impl FnMut(E) + 'static) {
    let cb = Closure::<dyn FnMut(E)>::new(f);
    let _ = target.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref());
    cb.forget();
}

fn install_input(app: &Rc<RefCell<App>>, canvas: &web_sys::HtmlCanvasElement) {
    let win: web_sys::EventTarget = window().into();
    {
        let app = app.clone();
        add_listener(&win, "keydown", move |e: web_sys::KeyboardEvent| {
            let code = e.code();
            let mut a = app.borrow_mut();
            a.audio.unlock();
            let mapped = map_key(&code);
            let tap = map_tap(&code);
            if let Some(k) = mapped {
                a.game.input.set_key(k, true);
            }
            if !e.repeat() {
                if let Some(t) = tap {
                    a.game.input.taps.push(t);
                }
            }
            if mapped.is_some() || code == "Space" {
                e.prevent_default();
            }
        });
    }
    {
        let app = app.clone();
        add_listener(&win, "keyup", move |e: web_sys::KeyboardEvent| {
            let code = e.code();
            if let Some(k) = map_key(&code) {
                app.borrow_mut().game.input.set_key(k, false);
                e.prevent_default();
            } else if code == "Space" {
                e.prevent_default();
            }
        });
    }
    let to_canvas = {
        let canvas = canvas.clone();
        move |e: &web_sys::MouseEvent| {
            let r = canvas.get_bounding_client_rect();
            (
                (e.client_x() as f64 - r.left()) * CANVAS_W as f64 / r.width().max(1.0),
                (e.client_y() as f64 - r.top()) * CANVAS_H as f64 / r.height().max(1.0),
            )
        }
    };
    {
        let app = app.clone();
        let to_canvas = to_canvas.clone();
        add_listener(&win, "mousemove", move |e: web_sys::MouseEvent| {
            let (x, y) = to_canvas(&e);
            let mut a = app.borrow_mut();
            a.game.input.mouse_x = x;
            a.game.input.mouse_y = y;
        });
    }
    {
        let app = app.clone();
        add_listener(&win, "mousedown", move |e: web_sys::MouseEvent| {
            let (x, y) = to_canvas(&e);
            let mut a = app.borrow_mut();
            a.audio.unlock();
            a.game.input.mouse_x = x;
            a.game.input.mouse_y = y;
            if e.button() == 0 {
                a.game.input.mouse_down = true;
            }
        });
    }
    {
        let app = app.clone();
        add_listener(&win, "mouseup", move |e: web_sys::MouseEvent| {
            if e.button() == 0 {
                app.borrow_mut().game.input.mouse_down = false;
            }
        });
    }
}

fn request_frame(f: &Closure<dyn FnMut(f64)>) {
    let _ = window().request_animation_frame(f.as_ref().unchecked_ref());
}

fn run_interactive(app: App, canvas: web_sys::HtmlCanvasElement) {
    let app = Rc::new(RefCell::new(app));
    install_input(&app, &canvas);
    {
        let mut a = app.borrow_mut();
        // Play the start-up sounds (background music); it actually starts on the first gesture.
        let sounds = std::mem::take(&mut a.game.sounds);
        a.audio.play(&sounds);
        a.last_tick = now_ms();
        a.title_t = a.last_tick;
    }
    let holder: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
    let h2 = holder.clone();
    *holder.borrow_mut() = Some(Closure::new(move |_ts: f64| {
        app.borrow_mut().frame(now_ms());
        if let Some(cb) = h2.borrow().as_ref() {
            request_frame(cb);
        }
    }));
    if let Some(cb) = holder.borrow().as_ref() {
        request_frame(cb);
    }
    // `holder` stays alive through the closure's own Rc clone.
    std::mem::forget(holder);
}

/// Resolves on the next macrotask without the 4 ms setTimeout clamp.
async fn yield_to_event_loop(channel: &web_sys::MessageChannel) {
    let port1 = channel.port1();
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let cb = Closure::once_into_js(move |_: JsValue| {
            let _ = resolve.call0(&JsValue::NULL);
        });
        port1.set_onmessage(Some(cb.unchecked_ref()));
    });
    let _ = channel.port2().post_message(&JsValue::NULL);
    let _ = JsFuture::from(promise).await;
}

fn work_done_promise(queue: &wgpu::Queue) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _| {
        queue.on_submitted_work_done(move || {
            let _ = resolve.call0(&JsValue::NULL);
        });
    })
}

async fn run_bench(mut app: App, cfg: Config) {
    let channel = web_sys::MessageChannel::new().expect("MessageChannel");
    let start = now_ms();
    let mut last_start = start;
    let mut measure_start: Option<f64> = None;
    let mut frame_times: Vec<f64> = Vec::new();
    let mut in_flight: VecDeque<js_sys::Promise> = VecDeque::new();
    let mut last_display = start;
    let mut n;
    let doc = window().document().expect("document");
    loop {
        let frame_start = now_ms();
        let ft = frame_start - last_start;
        last_start = frame_start;
        let t = ((frame_start - start) / 1000.0) as f32;

        // Simulation by the frame's real dt (fixed 60 Hz steps).
        app.acc += ft.min(250.0);
        while app.acc >= DT {
            app.acc -= DT;
            app.game.update();
            app.game.sounds.clear();
        }
        {
            let mut p = Painter::new(&mut app.scene, &app.atlas.uv);
            app.game.draw(&mut p, app.acc);
            stress_quads(&mut p, cfg.quads, t);
            n = p.n;
        }
        app.gpu.upload(&app.scene, n);

        // Two frames in flight.
        if in_flight.len() >= 2 {
            if let Some(p) = in_flight.pop_front() {
                let _ = JsFuture::from(p).await;
            }
        }
        app.gpu.render_offscreen(n);
        in_flight.push_back(work_done_promise(&app.gpu.queue));

        // Show something about once a second.
        if frame_start - last_display >= 1000.0 {
            last_display = frame_start;
            app.gpu.upload(&app.scene, n);
            app.gpu.render_surface(n);
        }

        if measure_start.is_none() {
            if (frame_start - start) / 1000.0 >= WARMUP_SECS {
                measure_start = Some(frame_start);
            }
        } else if let Some(ms) = measure_start {
            frame_times.push(ft);
            if (frame_start - ms) / 1000.0 >= cfg.seconds {
                break;
            }
        }
        yield_to_event_loop(&channel).await;
    }
    while let Some(p) = in_flight.pop_front() {
        let _ = JsFuture::from(p).await;
    }
    let extra = format!("\"api\":\"{}\",", app.gpu.api);
    let line = report_json("wasm", "offscreen", &extra, cfg.quads, cfg.seconds, &frame_times, &app.gpu.gpu_name)
        .unwrap_or_else(|| "{\"error\":\"no frames measured\"}".to_string());
    web_sys::console::log_1(&line.clone().into());
    if let Some(pre) = doc.get_element_by_id("bench-result") {
        let _ = pre.set_attribute("style", "display:block");
        pre.set_text_content(Some(&line));
    }
    doc.set_title(&format!("{} | bench done", app.title_prefix()));
}

async fn start_app() {
    let cfg = parse_config();
    let doc = window().document().expect("document");
    let canvas: web_sys::HtmlCanvasElement = doc
        .get_element_by_id("game")
        .and_then(|e| e.dyn_into().ok())
        .expect("canvas#game");
    canvas.set_width(CANVAS_W as u32);
    canvas.set_height(CANVAS_H as u32);

    let atlas = match atlas::build() {
        Ok(a) => a,
        Err(e) => return show_message(&format!("Atlas build failed: {e}")),
    };
    let capacity = SCENE_CAPACITY + if cfg.bench { cfg.quads } else { 0 };
    let gpu = match Gpu::new(canvas.clone(), capacity, &atlas.rgba).await {
        Ok(g) => g,
        Err(e) => {
            return show_message(&format!(
                "Furious Snake needs WebGPU or WebGL2, and neither could be started in this browser.\n\n{e}"
            ))
        }
    };
    web_sys::console::log_1(&format!("graphics backend: {} ({})", gpu.api, gpu.gpu_name).into());

    let game = if cfg.bench {
        Game::for_screen(ScreenKind::Race, cfg.level)
    } else {
        let seed = js_sys::Date::now() as u64;
        Game::with_progress(
            Progress::with_store(Some(Box::new(LocalStore))),
            Rng::new((seed ^ (seed >> 32)) as u32),
        )
    };
    let audio = Audio::new(cfg.mute || cfg.bench);
    let now = now_ms();
    let app = App {
        game,
        audio,
        atlas,
        gpu,
        scene: vec![Instance::default(); capacity],
        canvas: canvas.clone(),
        last_tick: now,
        acc: 0.0,
        title_t: now,
        title_frames: 0,
    };
    doc.set_title(&format!("{} | FPS 0", app.title_prefix()));
    let _ = &app.canvas;
    if cfg.bench {
        run_bench(app, cfg).await;
    } else {
        run_interactive(app, canvas);
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    wasm_bindgen_futures::spawn_local(start_app());
}
