//! Audio through HtmlAudioElement, mirroring the native rodio player. Browsers block sound
//! until the first user gesture, so everything starts on the first click or keypress.

use furious_core::sound::Sound;
use wasm_bindgen::prelude::*;
use web_sys::HtmlAudioElement;

const MUSIC_FILE: &str = "assets/music/background-theme.mp3";

pub struct Audio {
    enabled: bool,
    unlocked: bool,
    music_wanted: bool,
    music: Option<HtmlAudioElement>,
    ghost: Option<HtmlAudioElement>,
    ignore: Closure<dyn FnMut(JsValue)>,
}

fn file_for(s: Sound) -> Option<&'static str> {
    Some(match s {
        Sound::BlockCollision => "assets/sounds/error.mp3",
        Sound::Goalline => "assets/sounds/goal-line.mp3",
        Sound::StarPickUp => "assets/sounds/star.mp3",
        Sound::GainHeart => "assets/sounds/gain-heart.mp3",
        Sound::GhostPlay | Sound::GhostStop => "assets/sounds/ghost.mp3",
        Sound::MusicLoop | Sound::MusicStop => return None,
    })
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        Audio {
            enabled: !mute,
            unlocked: false,
            music_wanted: false,
            music: None,
            ghost: None,
            ignore: Closure::new(|_| {}),
        }
    }

    fn start(&self, el: &HtmlAudioElement) {
        if let Ok(p) = el.play() {
            let _ = p.catch(&self.ignore);
        }
    }

    fn start_music(&mut self) {
        if self.music.is_none() {
            if let Ok(m) = HtmlAudioElement::new_with_src(MUSIC_FILE) {
                m.set_loop(true);
                self.music = Some(m);
            }
        }
        if let Some(m) = &self.music {
            if m.paused() {
                self.start(m);
            }
        }
    }

    /// Called from the first keydown / mousedown.
    pub fn unlock(&mut self) {
        if self.unlocked || !self.enabled {
            return;
        }
        self.unlocked = true;
        if self.music_wanted {
            self.start_music();
        }
    }

    pub fn play(&mut self, events: &[Sound]) {
        if !self.enabled {
            return;
        }
        for &e in events {
            match e {
                Sound::MusicLoop => {
                    self.music_wanted = true;
                    if self.unlocked {
                        self.start_music();
                    }
                }
                Sound::MusicStop => {
                    self.music_wanted = false;
                    if let Some(m) = self.music.take() {
                        let _ = m.pause();
                    }
                }
                Sound::GhostStop => {
                    if let Some(g) = self.ghost.take() {
                        let _ = g.pause();
                    }
                }
                Sound::GhostPlay => {
                    if self.unlocked {
                        if let Some(g) = file_for(e).and_then(|f| HtmlAudioElement::new_with_src(f).ok()) {
                            self.start(&g);
                            self.ghost = Some(g);
                        }
                    }
                }
                other => {
                    if self.unlocked {
                        if let Some(s) = file_for(other).and_then(|f| HtmlAudioElement::new_with_src(f).ok()) {
                            self.start(&s);
                        }
                    }
                }
            }
        }
    }
}
