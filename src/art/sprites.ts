// Original 8-bit style pixel art. Each sprite is a character grid: "." is
// transparent, every other character indexes into the sprite's palette.
// No p5 here — this module is pure data so the tests can validate it in node.

export interface Sprite {
  rows: string[];
  palette: Record<string, string>;
}

// O outline, Y body, W highlight
const STAR_ROWS = [
  ".......OO.......",
  ".......OYO......",
  "......OYYO......",
  "......OYYYO.....",
  ".....OYYYYO.....",
  "OOOOOYYYYYYOOOOO",
  "OYYYYYYWWYYYYYYO",
  ".OYYYYYWWYYYYYO.",
  "..OYYYYYYYYYYO..",
  "...OYYYYYYYYO...",
  "....OYYYYYYO....",
  "...OYYYOOYYYO...",
  "..OYYYO..OYYYO..",
  ".OYYO......OYYO.",
  ".OYO........OYO.",
  "..O..........O..",
];

const HEART_ROWS = [
  "..OOO......OOO..",
  ".ORRROO..OORRRO.",
  "ORWWRRROORRRRRRO",
  "ORWWRRRRRRRRRRRO",
  "ORWRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  ".ORRRRRRRRRRRRO.",
  ".ORRRRRRRRRRRRO.",
  "..ORRRRRRRRRRO..",
  "...ORRRRRRRRO...",
  "....ORRRRRRO....",
  ".....ORRRRO.....",
  "......ORRO......",
  ".......OO.......",
  "................",
];

const GHOST_ROWS = [
  ".....OOOOOO.....",
  "...OOWWWWWWOO...",
  "..OWWWWWWWWWWO..",
  ".OWWWWWWWWWWWWO.",
  ".OWWEEWWWWEEWWO.",
  ".OWWEEWWWWEEWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWEEEEWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWWWWWWWWWWWWO.",
  ".OWOWWOWWOWWOWO.",
  ".OOO.OO.OO.OOOO.",
  "................",
];

// Tall sprite: carnivorous flower head over a stem.
const PLANT_ROWS = [
  ".....OOOOOO.....",
  "...OORRRRRROO...",
  "..ORRRRRRRRRRO..",
  ".ORRWWRRRRWWRRO.",
  ".ORRWWRRRRWWRRO.",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  "OWWOWWOWWOWWOWWO",
  "OWWOWWOWWOWWOWWO",
  "ORRRRRRRRRRRRRRO",
  "ORRRRRRRRRRRRRRO",
  ".ORRRRRRRRRRRRO.",
  "..ORRRRRRRRRRO..",
  "...OORRRRRROO...",
  ".....OOOOOO.....",
  ".......DD.......",
  ".......DD.......",
  "......DGGD......",
  "......DGGD......",
  "......DGGD......",
  "......DGGD......",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  ".....DGGGGD.....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "....DGGGGGGD....",
  "...DGGGGGGGGD...",
  "...DGGGGGGGGD...",
  "...DDDDDDDDDD...",
];

const WALL_ROWS = [
  "DDDDDDDDDDDDDDDD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DBBBBBBDBBBBBBBD",
  "DDDDDDDDDDDDDDDD",
  "DBBBDBBBBBBBDBBD",
  "DBBBDBBBBBBBDBBD",
  "DDDDDDDDDDDDDDDD",
];

const TETRIS_ROWS = [
  "DDDDDDDDDDDDDDDD",
  "DLLLLLLLLLLLLLLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DLGGGGGGGGGGGGLD",
  "DDGGGGGGGGGGGGDD",
  "DDDDDDDDDDDDDDDD",
];

// Checkered finish flag on a pole.
const WIN_ROWS = [
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PPWWKKWWKKWWKKWW",
  "PPKKWWKKWWKKWWKK",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
  "PP..............",
];

export const SPRITES: Record<string, Sprite> = {
  star: {
    rows: STAR_ROWS,
    palette: { O: "#4a2c00", Y: "#ffd93b", W: "#fff6a8" },
  },
  // Twinkle frame: same grid, hotter palette.
  starBright: {
    rows: STAR_ROWS,
    palette: { O: "#4a2c00", Y: "#ffe873", W: "#ffffff" },
  },
  heart: {
    rows: HEART_ROWS,
    palette: { O: "#7a0b1c", R: "#e8384f", W: "#ff9aa8" },
  },
  ghost: {
    rows: GHOST_ROWS,
    palette: { O: "#1a1a2e", W: "#f4f4f4", E: "#1a1a2e" },
  },
  plant: {
    rows: PLANT_ROWS,
    palette: {
      O: "#4a0d16",
      R: "#e8384f",
      W: "#ffffff",
      D: "#14532d",
      G: "#2fa14a",
    },
  },
  wall: {
    rows: WALL_ROWS,
    palette: { D: "#5a3218", B: "#9a5b2d" },
  },
  tetris: {
    rows: TETRIS_ROWS,
    palette: { D: "#3d7a06", G: "#7ed321", L: "#b8f04a" },
  },
  win: {
    rows: WIN_ROWS,
    palette: { P: "#8a6b3d", K: "#1a1a1a", W: "#f4f4f4" },
  },
};
