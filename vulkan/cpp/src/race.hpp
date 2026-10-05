#pragma once
#include <map>
#include <string>
#include <vector>

enum class RaceReason { None, Finish, FellBehind, NoLives, OpponentOut };

class RaceManager {
public:
    RaceManager(const std::vector<int>& playerNumbers, double startX, double finishX);

    void tick(double dtMs);
    void setHeadX(int pn, double x) { headX_[pn] = x; }
    double progress(int pn) const;
    double elapsedMs() const { return elapsed_; }
    bool isOver() const { return winner_ != 0; }
    void declareWinner(int pn, RaceReason reason);
    int winner() const { return winner_; }  // 0 = none
    RaceReason winReason() const { return reason_; }

private:
    double startX_, finishX_;
    std::map<int, double> headX_;
    double elapsed_ = 0;
    int winner_ = 0;
    RaceReason reason_ = RaceReason::None;
};

std::string formatTime(double ms);
