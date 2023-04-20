import React, { createContext, useState, Dispatch, SetStateAction, useEffect } from 'react';
import { Map } from 'immutable';
import { useAppDispatch, useAppSelector } from '../hooks/redux';
import { JoypadInput, inputPressed, inputReleased } from '../reducers/joypadReducer';


const JoypadContext = createContext<{
    joypadMap: Map<string, number>, 
    setJoypadMap: Dispatch<SetStateAction<Map<string, number>>>
}>({
    joypadMap: Map(),
    setJoypadMap: () => {
        console.warn("trying to remap keys before map was initialized");
    },
});

const mapKey = (map: Map<string, JoypadInput>, key: string): JoypadInput | undefined => {
    return map.get(key);
}

const Joypad = (props: {
    children?: React.ReactElement
}) => {
    const dispatch = useAppDispatch();
    const [joypadMap, setJoypadMap] = useState(Map({
        1: JoypadInput.A,
        2: JoypadInput.B,
        3: JoypadInput.Start,
        4: JoypadInput.Select,
        ArrowLeft: JoypadInput.Left,
        ArrowRight: JoypadInput.Right,
        ArrowUp: JoypadInput.Up,
        ArrowDown: JoypadInput.Down,
    }));

    const handleKeydown = (e: KeyboardEvent) => {
        const input = mapKey(joypadMap, e.key);
        if (input !== undefined) {
            e.preventDefault();
            dispatch(inputPressed(input));
        }
    };

    const handleKeyup = (e: KeyboardEvent) => {
        const input = mapKey(joypadMap, e.key);
        if (input !== undefined) {
            e.preventDefault();
            dispatch(inputReleased(input));
        }
    };

    useEffect(() => {
        // set up keyboard listeners
        window.addEventListener("keydown", handleKeydown);
        window.addEventListener("keyup", handleKeyup);
        return () => {
            window.removeEventListener("keydown", handleKeydown);
            window.removeEventListener("keyup", handleKeyup);
        }
    }, [])

    return (
        <JoypadContext.Provider value={{joypadMap, setJoypadMap}}>
            <div>
                {props.children}
            </div>
        </JoypadContext.Provider>
    );
}

export default Joypad;
export { JoypadContext };