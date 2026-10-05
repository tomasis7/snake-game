#include "audio.hpp"

#include <miniaudio.h>

#include <array>
#include <cstdio>

namespace {
const char* fileFor(Sfx s) {
    switch (s) {
        case Sfx::GainHeart: return "gain-heart.mp3";
        case Sfx::LostHeart: return "lost-heart.mp3";
        case Sfx::GameOver: return "game-over.mp3";
        case Sfx::Ghost: return "ghost.mp3";
        case Sfx::StarPickUp: return "star.mp3";
        case Sfx::Winner: return "winner.mp3";
        case Sfx::BlockCollision: return "error.mp3";
        case Sfx::WallCollision: return "shutdown-sound.mp3";
        case Sfx::GoalLine: return "goal-line.mp3";
    }
    return "";
}
}  // namespace

struct Audio::Impl {
    ma_engine engine{};
    bool engineOk = false;
    std::string soundsDir;
    std::string musicPath;
    ma_sound music{};
    bool musicLoaded = false;
    ma_sound ghost{};  // the one sound the TS stops explicitly
    bool ghostLoaded = false;
};

std::unique_ptr<Audio> Audio::create(const std::string& dir) {
    std::unique_ptr<Audio> a(new Audio());
    a->impl_ = std::make_unique<Impl>();
    Impl& m = *a->impl_;
    if (ma_engine_init(nullptr, &m.engine) != MA_SUCCESS) {
        std::fprintf(stderr, "warning: no audio device available, continuing without sound\n");
        return nullptr;
    }
    m.engineOk = true;
    m.soundsDir = dir + "/sounds/";
    m.musicPath = dir + "/music/background-theme.mp3";
    if (ma_sound_init_from_file(&m.engine, m.musicPath.c_str(), MA_SOUND_FLAG_STREAM, nullptr, nullptr, &m.music) ==
        MA_SUCCESS) {
        m.musicLoaded = true;
        ma_sound_set_looping(&m.music, MA_TRUE);
    }
    std::string g = m.soundsDir + fileFor(Sfx::Ghost);
    if (ma_sound_init_from_file(&m.engine, g.c_str(), 0, nullptr, nullptr, &m.ghost) == MA_SUCCESS)
        m.ghostLoaded = true;
    return a;
}

Audio::~Audio() {
    if (!impl_) return;
    if (impl_->ghostLoaded) ma_sound_uninit(&impl_->ghost);
    if (impl_->musicLoaded) ma_sound_uninit(&impl_->music);
    if (impl_->engineOk) ma_engine_uninit(&impl_->engine);
}

void Audio::play(Sfx s) {
    Impl& m = *impl_;
    if (s == Sfx::Ghost) {
        if (m.ghostLoaded) {
            ma_sound_seek_to_pcm_frame(&m.ghost, 0);
            ma_sound_start(&m.ghost);
        }
        return;
    }
    std::string path = m.soundsDir + fileFor(s);
    ma_engine_play_sound(&m.engine, path.c_str(), nullptr);  // overlapping one-shot, like p5 play()
}

void Audio::stop(Sfx s) {
    Impl& m = *impl_;
    if (s == Sfx::Ghost && m.ghostLoaded) ma_sound_stop(&m.ghost);
}

void Audio::loopMusic() {
    Impl& m = *impl_;
    if (!m.musicLoaded) return;
    ma_sound_seek_to_pcm_frame(&m.music, 0);
    ma_sound_start(&m.music);
}

void Audio::stopMusic() {
    if (impl_->musicLoaded) ma_sound_stop(&impl_->music);
}

bool Audio::musicPlaying() const { return impl_->musicLoaded && ma_sound_is_playing(&impl_->music); }
