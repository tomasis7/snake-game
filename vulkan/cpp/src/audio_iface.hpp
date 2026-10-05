#pragma once

// The TS `sounds.*` set, plus the background music hooks the game logic uses.
enum class Sfx { GainHeart, LostHeart, GameOver, Ghost, StarPickUp, Winner, BlockCollision, WallCollision, GoalLine };

class IAudio {
public:
    virtual ~IAudio() = default;
    virtual void play(Sfx s) = 0;
    virtual void stop(Sfx s) = 0;
    virtual void loopMusic() = 0;
    virtual void stopMusic() = 0;
    virtual bool musicPlaying() const = 0;
};

// Silent implementation for bench, screenshots, tests and --mute.
class NullAudio : public IAudio {
public:
    void play(Sfx) override {}
    void stop(Sfx) override {}
    void loopMusic() override { music_ = true; }
    void stopMusic() override { music_ = false; }
    bool musicPlaying() const override { return music_; }

private:
    bool music_ = false;
};
