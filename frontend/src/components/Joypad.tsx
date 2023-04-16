import React, { useEffect, createContext, useState, Dispatch, SetStateAction } from 'react';
import { Map } from 'immutable';
import JoypadDisplay from './JoypadDisplay';
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
    focusRef: React.RefObject<HTMLElement>,
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

    if (props.focusRef.current) {
        // set up keyboard listeners
        props.focusRef.current.onkeydown = (e: KeyboardEvent) => {
            const input = mapKey(joypadMap, e.key);

            if (input !== undefined) {
                e.preventDefault();
                console.log('pressing', input);
                dispatch(inputPressed(input));
            }
        }
        props.focusRef.current.onkeyup = (e: KeyboardEvent) => {
            const input = mapKey(joypadMap, e.key);

            if (input !== undefined) {
                e.preventDefault();
                dispatch(inputReleased(input));
            }
        }
    }

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