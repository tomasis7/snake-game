#pragma once
#include <memory>
#include <string>

#include "audio_iface.hpp"

// miniaudio-backed player for the TS game's sound effects and looping background music.
class Audio : public IAudio {
public:
    // Returns nullptr (after printing one warning) when no audio device can be opened.
    static std::unique_ptr<Audio> create(const std::string& publicAssetDir);
    ~Audio() override;

    void play(Sfx s) override;
    void stop(Sfx s) override;
    void loopMusic() override;
    void stopMusic() override;
    bool musicPlaying() const override;

private:
    Audio() = default;
    struct Impl;
    std::unique_ptr<Impl> impl_;
};
