import * as wasm from "gameboy-emulator";

let gameboy = wasm.GameBoyState.new_web();
console.log(gameboy);
gameboy.load_zelda();
console.log(gameboy.tick_for_frame());
console.log(gameboy.get_web_screen());