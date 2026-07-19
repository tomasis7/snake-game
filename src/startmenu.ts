import { GameScreen } from "./gamescreen";
import { Button } from "./button";
import { InteractionScreen } from "./interactionscreen";
import { GameMode } from "./progress";

export class StartMenu extends GameScreen {
  startGameButton: Button;
  onePlayerButton: Button;
  twoPlayerButton: Button;
  howToPlayButton: Button;
  selectedMode: GameMode = "onePlayer";

  constructor() {
    super();
    this.startGameButton = new Button(
      "Start Game",
      createVector(width / 2, height / 2 + 125),
      "#515151",
      createVector(350, 50),
      "#45FF8C"
    );

    this.onePlayerButton = new Button(
      "1 Player vs Robot",
      createVector(width / 2, height / 2 - 100),
      "#515151",
      createVector(420, 50),
      "#00FFFF"
    );

    this.twoPlayerButton = new Button(
      "2 Players",
      createVector(width / 2, height / 2 - 25),
      "#515151",
      createVector(420, 50),
      "#FF00FF"
    );

    this.howToPlayButton = new Button(
      "How to play",
      createVector(width / 2, height - 100),
      "#515151",
      createVector(380, 50),
      "#FFFFFF"
    );
  }

  update(): void {
    if (this.onePlayerButton.isClicked()) {
      this.selectedMode = "onePlayer";
    }

    if (this.twoPlayerButton.isClicked()) {
      this.selectedMode = "twoPlayer";
    }

    if (this.startGameButton.isClicked()) {
      userStartAudio();
      if (!music.backgroundMusic.isPlaying()) {
        music.backgroundMusic.loop();
      }
      game.startRun(this.selectedMode);
    }

    if (this.howToPlayButton.isClicked()) {
      game.changeScreen(new InteractionScreen());
    }
  }

  draw(): void {
    background("black");

    push();
    fill("#45FF8C");
    textAlign(CENTER, CENTER);
    textFont(customFont);
    textSize(42);
    text("Furious Snake", width / 2, height / 4 - 100);

    this.onePlayerButton.backgroundColor =
      this.selectedMode === "onePlayer" ? "white" : "#515151";
    this.twoPlayerButton.backgroundColor =
      this.selectedMode === "twoPlayer" ? "white" : "#515151";

    fill("#45FF8C");
    textSize(32);
    text("SELECT MODE", width / 2, height / 4);

    this.startGameButton.draw();
    this.onePlayerButton.draw();
    this.twoPlayerButton.draw();
    this.howToPlayButton.draw();
    pop();
  }
}
