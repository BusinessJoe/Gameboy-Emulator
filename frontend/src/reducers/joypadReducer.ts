import { Set } from "immutable";
import { AnyAction } from "@reduxjs/toolkit";

enum JoypadInput {
    A = 0,
    B = 1,
    Start = 2,
    Select = 3,
    Left = 4,
    Right = 5,
    Up = 6,
    Down = 7,
}

interface JoypadState {
    pressed?: JoypadInput,
    released?: JoypadInput,
    current: Set<JoypadInput>,
}

const initialState: JoypadState = {
    pressed: undefined,
    released: undefined,
    current: Set(),
};

const joypadReducer = (state = initialState, action: AnyAction) => {
    switch (action.type) {
        case 'inputPressed': {
            return {
                pressed: action.payload,
                // reset released if the input stored there was just pressed
                released: (action.payload === state.released) ? undefined : state.released,
                current: state.current.add(action.payload),
            }
        }
        case 'inputReleased': {
            return {
                // reset pressed if the input stored there was just released
                pressed: (action.payload === state.pressed) ? undefined : state.pressed,
                released: action.payload,
                current: state.current.delete(action.payload),
            }
        }
        default: {
            console.error("invalid action: ", action);
            return state
        }
    }
}

const inputPressed = (input: JoypadInput) => ({ type: 'inputPressed', payload: input });
const inputReleased = (input: JoypadInput) => ({ type: 'inputReleased', payload: input });

export default joypadReducer;
export { JoypadInput, inputPressed, inputReleased };