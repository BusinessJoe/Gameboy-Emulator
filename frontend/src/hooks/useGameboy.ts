import { useCallback, useEffect, useState } from "react";
import initWasm, { GameBoyState } from "gameboy_emulator";
import useAudio from "./useAudio";
import { load_ram, save_ram } from "../utils/database";


const useGameboy = (romData: Uint8Array | undefined) => {
    const [gameboy, setGameboy] = useState<GameBoyState>();
    const [screen, setScreen] = useState<Uint8Array | undefined>(undefined);

    /// Ticks the gameboy for one frame, updating the screen and returning the audio produced during the frame
    const getNextFrameAudio = useCallback(() => {
        if (gameboy === undefined) {
            throw new TypeError('Gameboy is undefined');
        }

        const startTime = performance.now();

        gameboy.tick_for_frame();
        setScreen(gameboy.get_web_screen());
        const frameAudio = gameboy.get_queued_audio();

        const endTime = performance.now();
        if (endTime - startTime > 1000/60) {
            console.warn(`Frame took too long: ${endTime - startTime} ms`);
        }

        return frameAudio;
    }, [gameboy]);

    const { init: initAudio, resume: resumeAudioPlayback } = useAudio(getNextFrameAudio);

    const loadSave = (gameboy: GameBoyState, identifier: string) => {
        load_ram(identifier)
        .then(ram => {
            console.log('Save found for game ', identifier, ram);
            if (gameboy.load_save(ram)) {
                console.log('Loaded save');
            } else {
                console.error('Failed to load save');
            }
            })
        .catch(() => {
            // no save found
            console.log('No save found for game ', identifier);
        })
    }

    const saveRam = useCallback(async () => {
        const identifier = gameboy?.game_name();
        const saveData = gameboy?.get_save();

        if (identifier === undefined || saveData === undefined) {
            throw new TypeError('Identifier or save data is undefined');
        }

        await save_ram(identifier, saveData);
    }, [gameboy]);

    /// Initialization
    useEffect(() => {
        if (romData) {
            console.log('Initializing wasm');
            initWasm()
                .then(async () => {
                    const gameboy = GameBoyState.new();
                    gameboy.load_rom_web(romData);
                    setScreen(gameboy.get_web_screen());
                    setGameboy(gameboy);
                    const identifier = gameboy.game_name();
                    if (identifier !== undefined) {
                        loadSave(gameboy, identifier);
                    }
                    console.log('Initialized wasm');
                })
                .catch((e: Error) => {
                    console.error('Failed to initialize wasm:', e);
                })
                .then(async () => {
                    await initAudio();
                    console.log('Initialized audio');
                })
                .catch((e: Error) => {
                    console.error('Failed to initialize audio:', e);
                });

            return () => {
                // clean up audio
                // clean up gameboy
            }
        }
    }, [romData, initAudio])

    useEffect(() => {
        if (gameboy !== undefined) {
            console.log('Resuming audio playback');
            resumeAudioPlayback().catch(e => console.error('Failed to resume audio playback:', e));
        }
    }, [gameboy, resumeAudioPlayback])



    return { gameboy, screen, saveRam };
}

export default useGameboy;