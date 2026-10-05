#pragma once
#include <vector>

#include "entities.hpp"
#include "pathfinding.hpp"
#include "player.hpp"
#include "rng.hpp"

struct RobotContext {
    const std::vector<Entity>* entities = nullptr;
    double cameraOffset = 0;  // the kill line
    std::vector<const std::vector<Vec2>*> otherTrails;
};

class RobotPlayer : public Player {
public:
    RobotPlayer(Vec2 position, int playerNumber, Color fill, Color stroke, double mistakeChance, Rng& rng)
        : Player(position, playerNumber, fill, stroke), mistakeChance_(mistakeChance), rng_(rng) {}

    void setContext(const RobotContext& c) {
        context_ = c;
        hasContext_ = true;
    }

protected:
    void handleInput(const KeyState& keys, double dt) override;

private:
    void buildWorld(AIWorld& world, std::vector<Pickup>& pickups) const;

    double mistakeChance_;
    Rng& rng_;
    RobotContext context_;
    bool hasContext_ = false;
    double thinkTimer_ = 0;
};
