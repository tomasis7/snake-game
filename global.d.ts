// This file will add both p5 instance and global intellisense.
// It makes sure that all the global p5 functions are available to typescript

import module = require("p5");
import * as p5Global from "p5/global";
type p5GlobalInstance = typeof p5Global;

export = module;
export as namespace p5;
declare global {
  interface Window {
    p5: typeof module;
  }

  function loadSound(
    path: string | string[] | p5.File,
    successCallback?: () => void,
    errorCallback?: (err: unknown) => void,
    loadingCallback?: (percentageLoaded: number) => void
  ): p5.SoundFile;

  // p5.sound global; returns the Web Audio context p5 already manages.
  function getAudioContext(): AudioContext;

  // Shared game globals assigned by main.ts
  var game: any;
  var images: any;
  var sounds: any;
  var music: any;
  var customFont: any;
}
