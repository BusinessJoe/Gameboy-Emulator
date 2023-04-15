import { useContext, useEffect, useState } from "react";
import GameboyContext from "./GameboyContext";
import { save_ram } from "../utils/database";

const Save = () => {
    const [prevTime, setPrevTime] = useState<number>();
    const [recentlySaved, setRecentlySaved] = useState(false);
    const { gameboy } = useContext(GameboyContext);

    /// set up 1 second interval to update time
    useEffect(() => {
        const interval = setInterval(() => {
            setPrevTime( prevTime => {
                if (prevTime === undefined) {
                    return 0;
                } else {
                    return prevTime + 1;
                }
            })
        }, 1000);

        return () => clearInterval(interval);
    }, [])

    const save = () => {
        const identifier = gameboy.game_name();
        const ram = gameboy.get_save();
        if (identifier !== undefined && ram !== undefined) {
            save_ram(identifier, ram)
                .then(() => {
                    setPrevTime(0);
                })
                .catch((e) => {
                    console.error("Failed to save", e);
                })
        }

        setRecentlySaved(true);
    }

    // reset recently saved to false after a duration
    useEffect(() => {
        let timeout: NodeJS.Timeout | undefined;
        if (recentlySaved) {
            timeout = setTimeout(() => setRecentlySaved(false), 2000);
        }
        return () => clearTimeout(timeout);
    }, [recentlySaved])

    const handleClick = () => {
        save()
    }

    return (
        <div>
            <button onClick={handleClick}>Save</button> {recentlySaved && <p>Saved!</p>}
        </div>
    );
}

export default Save;