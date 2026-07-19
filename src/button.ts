// Button Class
export class Button {
  text: string;
  position: p5.Vector;
  backgroundColor: string;
  size: p5.Vector;
  color: string;
  private pressHandled: boolean;

  constructor(
    text: string,
    position: p5.Vector,
    backgroundColor: string,
    size: p5.Vector,
    color: string
  ) {
    this.text = text;
    this.position = position;
    this.backgroundColor = backgroundColor;
    this.size = size;
    this.color = color;
    // A press already in progress when this button is created (e.g. the click
    // that switched screens) must not count as a click on this button.
    this.pressHandled = mouseIsPressed;
  }

  draw(): void {
    push();
    fill(this.backgroundColor);
    rectMode(CENTER);
    rect(this.position.x, this.position.y, this.size.x, this.size.y);
    fill(this.color);
    textAlign(CENTER, CENTER);
    text(this.text, this.position.x, this.position.y);
    pop();
  }

  isClicked(): boolean {
    if (!mouseIsPressed) {
      this.pressHandled = false;
      return false;
    }
    if (this.pressHandled) {
      return false;
    }
    const inside =
      mouseX > this.position.x - this.size.x / 2 &&
      mouseX < this.position.x + this.size.x / 2 &&
      mouseY > this.position.y - this.size.y / 2 &&
      mouseY < this.position.y + this.size.y / 2;
    if (inside) {
      this.pressHandled = true;
      return true;
    }
    return false;
  }
}
