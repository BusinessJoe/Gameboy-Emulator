import * as wasm from "gameboy-emulator";

const canvas = document.getElementById("screen");

const ctx = canvas.getContext("2d");
ctx.imageSmoothingEnabled = false;

let gameboy = wasm.GameBoyState.new_web();

let joypad_inputs = {
    a: false,
    b: false,
    start: false,
    select: false,
    left: false,
    right: false,
    up: false,
    down: false,
};

for (let [type, down] of [['keydown', true], ['keyup', false]]) {
    document.addEventListener(type, (e) => {
        console.log(e.code);
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
            console.log(key);
        }
    });
}

async function run_gameboy() {
    gameboy.load_zelda();

    while (true) {
        await update_frame(gameboy);   
        await new Promise(r => setTimeout(r, 0));
    }
}

async function update_frame(gameboy) {
    const cycles = gameboy.tick_for_frame();
    console.log('elapsed for: %d cycles', cycles);
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


run_gameboy();