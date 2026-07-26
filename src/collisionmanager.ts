import { Entity } from "./entity";
import { Player } from "./player";
import { Ghost } from "./ghost";
import { TetrisBlock } from "./tetrisBlocks";
import { Block } from "./block";
import { Star } from "./star";
import { Heart } from "./heart";
import { Plant } from "./plant";
import { WinBlock } from "./winBlock";
import { Effects } from "./effects/effects";

export class CollisionManager {
  players: Player[];
  entities: Entity[];
  private removeEntityCallback: (entity: Entity) => void;
  private onFinish: (playerNumber: number) => void;
  private onEliminate: (playerNumber: number) => void;
  private effects: Effects;

  constructor(
    players: Player[],
    entities: Entity[],
    removeEntityCallback: (entity: Entity) => void,
    onFinish: (playerNumber: number) => void,
    onEliminate: (playerNumber: number) => void,
    effects: Effects
  ) {
    this.players = players;
    this.entities = entities;
    this.removeEntityCallback = removeEntityCallback;
    this.onFinish = onFinish;
    this.onEliminate = onEliminate;
    this.effects = effects;
  }

  // A hazard costs one life and stuns, throttled by the collision cooldown so a
  // single overlap can't drain lives every frame. Phase mode (Phase 2) will set
  // canPassThroughObstacles; today it is always false.
  private handleHazard(player: Player): void {
    const now = Date.now();
    if (now - player.lastCollisionTime < player.collisionCooldown) return;
    player.lastCollisionTime = now;

    if (player.canPassThroughObstacles) return;

    sounds.blockCollision.play();
    this.effects.shake(10);
    this.effects.flash("#ff2d55");
    this.effects.burst(player.trail[0].x, player.trail[0].y, "#ff2d55", 10);
    player.applyStun(600);
    player.isColliding = true;
    player.lives -= 1;

    if (player.lives <= 0) {
      player.lives = 0;
      if (music.backgroundMusic.isPlaying()) {
        music.backgroundMusic.stop();
      }
      this.onEliminate(player.getPlayerNumber());
    }
  }

  private handleFinish(player: Player): void {
    sounds.goalline.play();
    this.effects.flash("#45FF8C");
    this.onFinish(player.getPlayerNumber());
  }

  private handleStarCollision(star: Entity): void {
    if (star.isRemoved) return;
    sounds.starPickUp.play();
    this.effects.burst(star.position.x, star.position.y, "#ffd93b");
    star.isRemoved = true;
    this.removeEntityCallback(star);
  }

  private handleHeartCollision(player: Player, heart: Entity): void {
    if (heart.isRemoved) return;
    sounds.gainheart.play();
    this.effects.burst(heart.position.x, heart.position.y, "#e8384f");
    if (player.lives < player.maxLives) {
      player.lives += 1;
    }
    heart.isRemoved = true;
    this.removeEntityCallback(heart);
  }

  private handleGhostProximity(player: Player, ghost: Entity): void {
    const distance = dist(
      player.trail[0].x,
      player.trail[0].y,
      ghost.position.x,
      ghost.position.y
    );

    if (distance < 200) {
      if (!ghost.isSoundPlaying) {
        sounds.ghost.play();
        ghost.isSoundPlaying = true;
      } else {
        sounds.ghost.stop();
        ghost.isSoundPlaying = false;
      }
    }
  }

  checkCollision(): void {
    for (const player of this.players) {
      const head = player.trail[0];
      const headLeft = head.x;
      const headRight = head.x + player.size.x;
      const headTop = head.y;
      const headBottom = head.y + player.size.y;

      let hasCollision = false;

      for (const entity of this.entities) {
        if (entity instanceof Ghost) {
          this.handleGhostProximity(player, entity);
        }

        const entityLeft = entity.position.x;
        const entityRight = entity.position.x + entity.size.x;
        const entityTop = entity.position.y;
        const entityBottom = entity.position.y + entity.size.y;

        const isColliding =
          headRight > entityLeft &&
          headLeft < entityRight &&
          headBottom > entityTop &&
          headTop < entityBottom;

        if (isColliding) {
          hasCollision = true;

          if (!player.isColliding) {
            if (
              entity instanceof TetrisBlock ||
              entity instanceof Block ||
              entity instanceof Plant ||
              entity instanceof Ghost
            ) {
              this.handleHazard(player);
            } else if (entity instanceof Star) {
              this.handleStarCollision(entity);
            } else if (entity instanceof Heart) {
              this.handleHeartCollision(player, entity);
            } else if (entity instanceof WinBlock) {
              this.handleFinish(player);
            }

            break;
          }
        }

        if (!hasCollision) {
          player.isColliding = false;
        }
      }
    }
  }
}
