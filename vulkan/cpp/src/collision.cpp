#include "collision.hpp"

#include <cmath>

namespace {
const Color RED = hex("#ff2d55");
const Color GREEN = hex("#45FF8C");
const Color GOLD = hex("#ffd93b");
const Color HEART_RED = hex("#e8384f");
}  // namespace

void CollisionManager::handleHazard(Player& p, double now) {
    if (now - p.lastCollisionTime < p.collisionCooldown) return;
    p.lastCollisionTime = now;

    if (p.canPassThroughObstacles) return;

    audio_->play(Sfx::BlockCollision);
    effects_->shake(10);
    effects_->flash(RED);
    effects_->burst(p.trail[0].x, p.trail[0].y, RED, 10);
    p.applyStun(now, 600);
    p.isColliding = true;
    p.lives -= 1;

    if (p.lives <= 0) {
        p.lives = 0;
        if (audio_->musicPlaying()) audio_->stopMusic();
        onEliminate_(p.playerNumber());
    }
}

void CollisionManager::handleFinish(Player& p) {
    audio_->play(Sfx::GoalLine);
    effects_->flash(GREEN);
    onFinish_(p.playerNumber());
}

void CollisionManager::handleStar(Entity& star) {
    if (star.removed) return;
    audio_->play(Sfx::StarPickUp);
    effects_->burst(star.x, star.y, GOLD);
    star.removed = true;
}

void CollisionManager::handleHeart(Player& p, Entity& heart) {
    if (heart.removed) return;
    audio_->play(Sfx::GainHeart);
    effects_->burst(heart.x, heart.y, HEART_RED);
    if (p.lives < p.maxLives) p.lives += 1;
    heart.removed = true;
}

void CollisionManager::handleGhostProximity(Player& p, Entity& ghost) {
    double distance = std::hypot(p.trail[0].x - ghost.x, p.trail[0].y - ghost.y);
    if (distance < 200) {
        if (!ghost.soundPlaying) {
            audio_->play(Sfx::Ghost);
            ghost.soundPlaying = true;
        } else {
            audio_->stop(Sfx::Ghost);
            ghost.soundPlaying = false;
        }
    }
}

void CollisionManager::checkCollision(double now) {
    for (Player* pp : players_) {
        Player& player = *pp;
        const Vec2 head = player.trail[0];
        double headLeft = head.x, headRight = head.x + player.size.x;
        double headTop = head.y, headBottom = head.y + player.size.y;

        bool hasCollision = false;

        for (Entity& entity : *entities_) {
            if (entity.kind == EntityKind::Ghost) handleGhostProximity(player, entity);

            double entityLeft = entity.x, entityRight = entity.x + entity.w;
            double entityTop = entity.y, entityBottom = entity.y + entity.h;

            bool colliding = headRight > entityLeft && headLeft < entityRight && headBottom > entityTop &&
                             headTop < entityBottom;

            if (colliding) {
                hasCollision = true;
                if (!player.isColliding) {
                    switch (entity.kind) {
                        case EntityKind::Tetris:
                        case EntityKind::Block:
                        case EntityKind::Plant:
                        case EntityKind::Ghost: handleHazard(player, now); break;
                        case EntityKind::Star: handleStar(entity); break;
                        case EntityKind::Heart: handleHeart(player, entity); break;
                        case EntityKind::Win: handleFinish(player); break;
                    }
                    break;
                }
            }

            if (!hasCollision) player.isColliding = false;
        }
    }
}
