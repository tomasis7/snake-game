#pragma once
#include <functional>
#include <vector>

#include "audio_iface.hpp"
#include "effects.hpp"
#include "entities.hpp"
#include "player.hpp"

class CollisionManager {
public:
    // `entities` must outlive the manager and never reallocate. Removed entities stay in it (flagged),
    // matching the TS manager, which keeps iterating its original array after removals.
    CollisionManager(std::vector<Player*> players, std::vector<Entity>* entities, Effects* effects,
                     IAudio* audio, std::function<void(int)> onFinish, std::function<void(int)> onEliminate)
        : players_(std::move(players)), entities_(entities), effects_(effects), audio_(audio),
          onFinish_(std::move(onFinish)), onEliminate_(std::move(onEliminate)) {}

    void checkCollision(double now);

private:
    void handleHazard(Player& p, double now);
    void handleFinish(Player& p);
    void handleStar(Entity& star);
    void handleHeart(Player& p, Entity& heart);
    void handleGhostProximity(Player& p, Entity& ghost);

    std::vector<Player*> players_;
    std::vector<Entity>* entities_;
    Effects* effects_;
    IAudio* audio_;
    std::function<void(int)> onFinish_, onEliminate_;
};
