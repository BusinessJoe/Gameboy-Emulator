import * as wasm from "gameboy-emulator";

const scale = 5;
const canvas = document.getElementById("canvas");

const ctx = canvas.getContext("2d");
ctx.imageSmoothingEnabled = false;


async function run_gameboy() {
    let gameboy = wasm.GameBoyState.new_web();
    gameboy.load_zelda();
    
    while (true) {
        await update_frame(gameboy);   
        await new Promise(r => setTimeout(r, 1));
    }
}

async function update_frame(gameboy) {
    const cycles = gameboy.tick_for_frame();
    console.log('elapsed for: %d cycles', cycles);
    const screen_data = gameboy.get_web_screen();

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