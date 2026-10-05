#pragma once
#include <map>
#include <string>

enum class GameMode { OnePlayer, TwoPlayer };

struct BestResult {
    double bestMs = 0;
    bool isNewBest = false;
};

// Run-wide progression: mode, current level and best finish time per level, persisted as
// "L<n> <ms>" lines. An empty path keeps everything in memory.
class Progress {
public:
    explicit Progress(std::string path = "");

    static std::string defaultPath();  // ${XDG_DATA_HOME:-~/.local/share}/furious-snake-vulkan/best_times.txt

    void startRun(GameMode m);
    bool isLastLevel() const { return currentLevel >= 3; }
    bool getBestTime(int level, double& out) const;
    BestResult recordBestTime(int level, double timeMs);

    GameMode mode = GameMode::OnePlayer;
    int currentLevel = 1;

private:
    void load();
    void save() const;

    std::string path_;
    std::map<int, double> best_;
};
