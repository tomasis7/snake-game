//! Audio playback of the TS game's mp3 files via rodio (cargo feature `audio`).
//! Without the feature, or without an audio device, one warning is printed and the game
//! continues silently.

use crate::sound::Sound;

#[cfg(feature = "audio")]
mod imp {
    use super::Sound;
    use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
    use std::collections::HashMap;
    use std::io::Cursor;

    const SOUNDS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../public/assets/sounds");
    const MUSIC_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../public/assets/music/background-theme.mp3");

    pub struct Audio {
        // Kept alive for the lifetime of the audio output.
        _stream: Option<OutputStream>,
        sounds: HashMap<Sound, Vec<u8>>,
        music_bytes: Option<Vec<u8>>,
        music: Option<Sink>,
        ghost: Option<Sink>,
        active: bool,
    }

    fn file_for(s: Sound) -> Option<&'static str> {
        Some(match s {
            Sound::BlockCollision => "error.mp3",
            Sound::Goalline => "goal-line.mp3",
            Sound::StarPickUp => "star.mp3",
            Sound::GainHeart => "gain-heart.mp3",
            Sound::GhostPlay | Sound::GhostStop => "ghost.mp3",
            Sound::MusicLoop | Sound::MusicStop => return None,
        })
    }

    impl Audio {
        pub fn new(mute: bool) -> Audio {
            let mut a = Audio {
                _stream: None,
                sounds: HashMap::new(),
                music_bytes: None,
                music: None,
                ghost: None,
                active: false,
            };
            if mute {
                return a;
            }
            match OutputStreamBuilder::open_default_stream() {
                Ok(mut stream) => {
                    stream.log_on_drop(false);
                    a._stream = Some(stream);
                    a.active = true;
                }
                Err(e) => {
                    eprintln!("warning: no audio device ({e}); continuing without sound");
                    return a;
                }
            }
            for s in [Sound::BlockCollision, Sound::Goalline, Sound::StarPickUp, Sound::GainHeart, Sound::GhostPlay] {
                if let Some(f) = file_for(s) {
                    match std::fs::read(format!("{SOUNDS_DIR}/{f}")) {
                        Ok(b) => {
                            a.sounds.insert(s, b);
                        }
                        Err(e) => eprintln!("warning: cannot read {f}: {e}"),
                    }
                }
            }
            a.music_bytes = std::fs::read(MUSIC_FILE).ok();
            a
        }

        fn sink(&self, bytes: &[u8], looped: bool) -> Option<Sink> {
            let stream = self._stream.as_ref()?;
            let sink = Sink::connect_new(stream.mixer());
            let cursor = Cursor::new(bytes.to_vec());
            if looped {
                sink.append(Decoder::new_looped(cursor).ok()?);
            } else {
                sink.append(Decoder::new(cursor).ok()?);
            }
            Some(sink)
        }

        pub fn play(&mut self, events: &[Sound]) {
            if !self.active {
                return;
            }
            for &e in events {
                match e {
                    Sound::MusicLoop => {
                        if self.music.as_ref().map_or(true, |m| m.empty()) {
                            if let Some(b) = self.music_bytes.clone() {
                                self.music = self.sink(&b, true);
                            }
                        }
                    }
                    Sound::MusicStop => {
                        if let Some(m) = self.music.take() {
                            m.stop();
                        }
                    }
                    Sound::GhostStop => {
                        if let Some(g) = self.ghost.take() {
                            g.stop();
                        }
                    }
                    Sound::GhostPlay => {
                        if let Some(b) = self.sounds.get(&e).cloned() {
                            self.ghost = self.sink(&b, false);
                        }
                    }
                    other => {
                        if let Some(b) = self.sounds.get(&other).cloned() {
                            if let Some(s) = self.sink(&b, false) {
                                s.detach();
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "audio"))]
mod imp {
    use super::Sound;

    pub struct Audio;

    impl Audio {
        pub fn new(mute: bool) -> Audio {
            if !mute {
                eprintln!("warning: built without the `audio` cargo feature; continuing without sound");
            }
            Audio
        }

        pub fn play(&mut self, _events: &[Sound]) {}
    }
}

pub use imp::Audio;
