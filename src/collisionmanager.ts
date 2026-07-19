import { Entity } from "./entity";
import { Player } from "./player";
import { ScoreManager } from "./scoreManager";
import { Ghost } from "./ghost";
import { TetrisBlock } from "./tetrisBlocks";
import { Block } from "./block";
import { Star } from "./star";
import { Heart } from "./heart";
import { Plant } from "./plant";
import { WinBlock } from "./winBlock";
import { Effects } from "./effects/effects";
import { RobotPlayer } from "./robotplayer";
import { ComboTracker } from "./combo";
import { playComboBlip } from "./audio/sfx";

export class CollisionManager {
  players: Player[];
  entities: Entity[];
  scoreManager: ScoreManager;
  private removeEntityCallback: (entity: Entity) => void;
  private onLevelEnd: () => void;
  private effects: Effects;
  private combos: Map<number, ComboTracker> = new Map();

  constructor(
    players: Player[],
    entities: Entity[],
    scoreManager: ScoreManager,
    removeEntityCallback: (entity: Entity) => void,
    onLevelEnd: () => void,
    effects: Effects
  ) {
    this.players = players;
    this.entities = entities;
    this.scoreManager = scoreManager;
    this.removeEntityCallback = removeEntityCallback;
    this.onLevelEnd = onLevelEnd;
    this.effects = effects;
  }

  private handleTetrisCollision(player: Player): void {
    sounds.blockCollision.play();
    if (music.backgroundMusic.isPlaying()) {
      music.backgroundMusic.stop();
    }

    player.isColliding = true;
    this.effects.shake(16);
    this.effects.flash("#ff2d55");
    player.isMoving = false;
    console.log(`Player ${player.playerNumber} collided with a TetrisBlock.`);

    this.onLevelEnd();
  }

  private handleBlockCollision(player: Player): void {
    sounds.wallCollision.play();
    if (music.backgroundMusic.isPlaying()) {
      music.backgroundMusic.stop();
    }
    player.isColliding = true;
    this.effects.shake(16);
    this.effects.flash("#ff2d55");
    player.isMoving = false;
    console.log(`Player ${player.playerNumber} collided with a TetrisBlock.`);

    this.onLevelEnd();
  }

  private handleWinBlockCollision(player: Player): void {
    sounds.goalline.play();
    if (music.backgroundMusic.isPlaying()) {
      music.backgroundMusic.stop();
    }

    player.isColliding = true;
    this.effects.flash("#45FF8C");
    player.isMoving = false;
    console.log(`Player ${player.playerNumber} won!`);
    this.onLevelEnd();
  }

  private static readonly TAUNTS = ["MINE!", "TOO SLOW!", "GOT IT!", "NICE TRY!"];

  // The robot gloats only when it stole something the human was closing in on.
  private maybeTaunt(collector: Player, pickup: Entity): void {
    if (!(collector instanceof RobotPlayer)) return;

    const human = this.players.find((p) => !(p instanceof RobotPlayer));
    if (!human) return;

    const distance = dist(
      human.trail[0].x,
      human.trail[0].y,
      pickup.position.x,
      pickup.position.y
    );
    if (distance > 4 * 32) return;

    const taunt =
      CollisionManager.TAUNTS[
        Math.floor(Math.random() * CollisionManager.TAUNTS.length)
      ];
    this.effects.floatText(
      pickup.position.x,
      pickup.position.y - 24,
      taunt,
      "#FF00FF"
    );
  }

  // Scores a pickup at the player's current combo tier and shows the chain.
  private awardPickup(player: Player, pickup: Entity, basePoints: number): void {
    let combo = this.combos.get(player.getPlayerNumber());
    if (!combo) {
      combo = new ComboTracker();
      this.combos.set(player.getPlayerNumber(), combo);
    }

    const multiplier = combo.registerPickup(Date.now());
    this.scoreManager.updateScore(
      player.getPlayerNumber(),
      basePoints * multiplier
    );

    const label =
      multiplier > 1
        ? `+${basePoints * multiplier}  x${multiplier}`
        : `+${basePoints}`;
    this.effects.floatText(
      pickup.position.x,
      pickup.position.y - 8,
      label,
      multiplier > 1 ? "#FDD03C" : "#FFFFFF"
    );

    if (multiplier > 1) {
      playComboBlip(multiplier);
    }
  }

  // A hit breaks the chain.
  private breakCombo(player: Player): void {
    this.combos.get(player.getPlayerNumber())?.reset();
  }

  private handleStarCollision(player: Player, star: Entity): void {
    if (star.isRemoved) return;
    sounds.starPickUp.play();
    player.isColliding = true;

    player.grow(1);
    this.effects.burst(star.position.x, star.position.y, "#ffd93b");
    this.maybeTaunt(player, star);
    this.awardPickup(player, star, 100);

    player.scoreMultiplier = 2;

    const scoreInterval = setInterval(() => {
      this.scoreManager.updateScore(
        player.getPlayerNumber(),
        50 * player.scoreMultiplier
      );
    }, 1000);

    setTimeout(() => {
      player.scoreMultiplier = 1;
      clearInterval(scoreInterval);
      console.log(`Player ${player.playerNumber}'s score multiplier reset.`);
    }, 10000);

    star.isRemoved = true;
    this.removeEntityCallback(star);
    console.log(`Star entity removed:`, star);
  }

  private handleHeartCollision(player: Player, heart: Entity): void {
    if (heart.isRemoved) return;

    sounds.gainheart.play();
    player.isColliding = true;
    console.log(`Player ${player.playerNumber} collected a Heart!`);

    player.grow(1);
    this.effects.burst(heart.position.x, heart.position.y, "#e8384f");
    this.maybeTaunt(player, heart);
    this.awardPickup(player, heart, 50);

    if (player.lives < player.maxLives) {
      player.lives += 1;
    }
    heart.isRemoved = true;
    this.removeEntityCallback(heart);
    console.log(`Heart entity removed:`, heart);
  }

  private handlePlantCollision(player: Player): void {
    const currentTime = Date.now();

    if (currentTime - player.lastCollisionTime < player.collisionCooldown) {
      return;
    }

    player.lastCollisionTime = currentTime;

    console.log(`Player lives before collision: ${player.lives}`);
    sounds.blockCollision.play();
    player.isColliding = true;
    this.effects.shake(10);
    this.breakCombo(player);
    this.effects.flash("#ff2d55");
    this.effects.burst(player.trail[0].x, player.trail[0].y, "#ff2d55", 10);

    player.lives -= 2;

    console.log(`Player lives after collision: ${player.lives}`);

    if (player.lives < 0) {
      player.lives = 0;
    }

    if (player.lives <= 0) {
      if (music.backgroundMusic.isPlaying()) {
        music.backgroundMusic.stop();
      }

      this.onLevelEnd();
    }
  }

  private handleGhostProximity(player: Player, ghost: Entity) {
    const distance = dist(
      player.trail[0].x,
      player.trail[0].y,
      ghost.position.x,
      ghost.position.y
    );

    if (distance < 200) {
      console.log("Ghost is near, playing sound...");

      if (!ghost.isSoundPlaying) {
        sounds.ghost.play();
        ghost.isSoundPlaying = true;
        console.log("Ghost sound started");
      } else {
        if (ghost.isSoundPlaying) {
          sounds.ghost.stop();
          ghost.isSoundPlaying = false;
          console.log("Ghost sound stopped");
        }
      }
    }
  }

  private handleGhostCollision(player: Player): void {
    const currentTime = Date.now();

    if (currentTime - player.lastCollisionTime > 2000) {
      player.isColliding = true;
      this.effects.shake(8);
      this.breakCombo(player);
      this.effects.burst(player.trail[0].x, player.trail[0].y, "#f4f4f4", 10);
      player.lives -= 1;

      if (player.lives < 0) {
        player.lives = 0;
      }

      if (player.lives === 0) {
        this.onLevelEnd();
      }

      this.scoreManager.updateScore(player.getPlayerNumber(), -5);

      player.lastCollisionTime = currentTime;
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
            if (entity instanceof TetrisBlock) {
              this.handleTetrisCollision(player);
            } else if (entity instanceof Block) {
              this.handleBlockCollision(player);
            } else if (entity instanceof Star) {
              this.handleStarCollision(player, entity);
            } else if (entity instanceof Heart) {
              this.handleHeartCollision(player, entity);
            } else if (entity instanceof Plant) {
              this.handlePlantCollision(player);
            } else if (entity instanceof Ghost) {
              this.handleGhostCollision(player);
            } else if (entity instanceof WinBlock) {
              this.handleWinBlockCollision(player);
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

  showPopupMessage(message: string, duration: number = 3000): void {
    const popup = document.createElement("div");
    popup.innerText = message;
    popup.style.position = "absolute";
    popup.style.top = "10px";
    popup.style.left = "50%";
    popup.style.transform = "translateX(-50%)";
    popup.style.backgroundColor = "rgba(0, 0, 0, 0.8)";
    popup.style.color = "white";
    popup.style.padding = "10px 20px";
    popup.style.borderRadius = "8px";
    popup.style.zIndex = "1000";
    document.body.appendChild(popup);

    setTimeout(() => {
      document.body.removeChild(popup);
    }, duration);
  }
}
