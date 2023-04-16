import init, { GameBoyState } from "gameboy_emulator";
import { ReactElement, createContext, useEffect, useState } from "react"

type GameboyContextValue = {
    gameboy: GameBoyState
}

// this is a not-so-great hack to avoid type errors with the default value
// since we're planning on always providing a default value this shouldn't matter
const GameboyContext = createContext<GameboyContextValue>({} as GameboyContextValue);

const GameboyProvider = (props: {
    children: ReactElement
}) => {
    const [gameboy, setGameboy] = useState<GameBoyState>();
    const [failed, setFailed] = useState(false);

    useEffect(() => {
        init().then(() => {
            console.log("initialized wasm");
            setGameboy(GameBoyState.new());
        }).catch(() => {
            setFailed(true);
        })
    }, [])

    if (failed) {
        return (
            <h1>Failed to initialize gameboy</h1>
        );
    }

    if (gameboy !== undefined) {

        const context = {
            gameboy
        }

        return (
            <GameboyContext.Provider value={context}>
                {props.children}
            </GameboyContext.Provider>
        );
    } else {
        return (
            <h1>Initializing gameboy</h1>
        )
    }
}

export default GameboyContext;
export { GameboyProvider };