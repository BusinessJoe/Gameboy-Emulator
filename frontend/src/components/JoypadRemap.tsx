import { useContext, useEffect, useState } from "react"
import { JoypadContext } from "./Joypad"
import { useKeyDown } from "../hooks/useKeyboardInput";

const JoypadRemap = () => {
    const { joypadMap, setJoypadMap } = useContext(JoypadContext);
    const [ pendingRemap, setPendingRemap ] = useState<string | undefined>(undefined);
    const { key } = useKeyDown();

    // each entry is [keycode, joypad input]
    const keycodeInputMap = joypadMap.sort().map(v => ['A', 'B', 'Start', 'Select', 'Left', 'Right', 'Up', 'Down'][v]);

    useEffect(() => {
        if (pendingRemap) {
            const value = joypadMap.get(pendingRemap);
            // current key must map to an existing value, and new key cannot be already used
            if (value !== undefined && joypadMap.get(key) === undefined) {
                setPendingRemap( undefined );
                setJoypadMap( joypadMap.delete(pendingRemap).set(key, value) );
            }
        }
    }, [key]);

    return (
        <div>
            <table>
                <tbody>
                    <tr>
                        {Array.from(keycodeInputMap.values()).map((entry, i) => <td key={i}>{entry}</td>)}
                    </tr>
                    <tr>
                        {Array.from(keycodeInputMap.keys()).map((entry, i) => 
                            <td key={i} onClick={() => setPendingRemap(entry)}>{entry}</td>
                        )}
                    </tr>
                </tbody>
            </table>
            {pendingRemap && 
                <p>Hit a key to remap</p>
            }
        </div>
    );
}

export default JoypadRemap;