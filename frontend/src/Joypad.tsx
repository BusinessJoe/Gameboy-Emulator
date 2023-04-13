import React, { useEffect, createContext, useState, Dispatch, SetStateAction } from 'react';
import { Map } from 'immutable';

const JoypadContext = createContext<{
    joypadMap: Map<string, number>, 
    setJoypadMap: Dispatch<SetStateAction<Map<string, number>>>
}>({
    joypadMap: Map(),
    setJoypadMap: () => {
        console.warn("trying to remap keys before map was initialized");
    },
});

const mapKey = (map: Map<string, number>, key: string): number | undefined => {
    return map.get(key);
}

const Joypad = (props: {
    focusRef: React.RefObject<HTMLElement>,
    onJoypadInput: (key: number, down: boolean) => void,
    children: React.ReactElement
}) => {
    const [joypadMap, setJoypadMap] = useState(Map({
        1: 0,
        2: 1,
        3: 2,
        4: 3,
        ArrowLeft: 4,
        ArrowRight: 5,
        ArrowUp: 6,
        ArrowDown: 7,
    }));

    if (props.focusRef.current) {
        // set up keyboard listeners
        props.focusRef.current.onkeydown = (e: KeyboardEvent) => {
            e.preventDefault();
            const key = mapKey(joypadMap, e.key);

            if (key !== undefined) {
                props.onJoypadInput(key, true);
            }
        }
        props.focusRef.current.onkeyup = (e: KeyboardEvent) => {
            e.preventDefault();
            const key = mapKey(joypadMap, e.key);

            if (key !== undefined) {
                props.onJoypadInput(key, false);
            }
        }
    }

    return (
        <JoypadContext.Provider value={{joypadMap, setJoypadMap}}>
            {props.children}
        </JoypadContext.Provider>
    );
}

export default Joypad;
export { JoypadContext };