import * as wasm from "gameboy-emulator";
import { save, load } from "./saves.js";

import "./styles.css"
const canvas = document.getElementById("screen");
const rom_upload = document.getElementById("rom-upload");
const rom_upload_wrapper = document.getElementById("rom-upload-wrapper");

const ctx = canvas.getContext("2d");
ctx.imageSmoothingEnabled = false;

let nextTimestamp;

let gameboy;
let frameCount = 0;

rom_upload.addEventListener('change', () => {
    const curFiles = rom_upload.files;

    if (curFiles.length === 0) {

    } else {
        const file = curFiles[0];
        const reader = new FileReader();
        reader.onload = (evt) => {
            const array = new Uint8ClampedArray(evt.target.result);
            gameboy = wasm.GameBoyState.new_web();
            console.log(array);
            gameboy.load_rom(array);
            load(gameboy);
            rom_upload_wrapper.style.display = "none";

            window.requestAnimationFrame(step);

            add_keyboard_listeners(gameboy);
        };
        reader.readAsArrayBuffer(file);
    }
});

function add_keyboard_listeners(gameboy) {
    for (let [type, down] of [['keydown', true], ['keyup', false]]) {
        document.addEventListener(type, (e) => {
            if (gameboy) {
                let key = -1;
                if (e.code == "Digit1") {
                    key = 0;
                } else if (e.code == "Digit2") {
                    key = 1;
                } else if (e.code == "Digit3") {
                    key = 2;
                } else if (e.code == "Digit4") {
                    key = 3;
                } else if (e.code == "ArrowLeft") {
                    key = 4;
                } else if (e.code == "ArrowRight") {
                    key = 5;
                } else if (e.code == "ArrowUp") {
                    key = 6;
                } else if (e.code == "ArrowDown") {
                    key = 7;
                }

                if (key >= 0) {
                    if (down) {
                        gameboy.press_key(key);
                    } else {
                        gameboy.release_key(key);
                    }
                }
            }
        });
    }
}

function step(timestamp) {
    if (nextTimestamp === undefined) {
        nextTimestamp = timestamp + 1000 / 60;
    }

    if (timestamp >= nextTimestamp) {
        // render current gameboy frame
        render_frame(gameboy);
        // tick gameboy for 1 frame
        gameboy.tick_for_frame();
        frameCount += 1;
        if (frameCount % (5 * 60) === 0) {
            save(gameboy);
        }

        nextTimestamp += 1000 / 60;

        if (nextTimestamp <= timestamp) {
            console.error("frame took too long");
            nextTimestamp = timestamp + 1000 / 60;
        }
    }

    window.requestAnimationFrame(step);
}

function render_frame(gameboy) {
    const screen_data = gameboy.get_web_screen();

    // allocate space for 4 color values (rgba) per screen pixel
    const arr = new Uint8ClampedArray(160 * 144 * 4);

    let color;
    for (let i = 0; i*4 < arr.length; i++) {
        switch (screen_data[i]) {
            case 0:
                color = [255, 255, 255];
                break;
            case 1:
                color = [200, 200, 200];
                break;
            case 2:
                color = [100, 100, 100];
                break;
            case 3:
                color = [0, 0, 0];
                break;
            default:
                color = [255, 0, 0];
                break;
        }

        arr[4*i + 0] = color[0];
        arr[4*i + 1] = color[1];
        arr[4*i + 2] = color[2];
        arr[4*i + 3] = 255;
    }

    const img_data = new ImageData(arr, 160, 144);
    ctx.putImageData(img_data, 0, 0);
}