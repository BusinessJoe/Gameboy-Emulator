import { combineReducers } from "@reduxjs/toolkit";
import joypadReducer from './joypadReducer';

const rootReducer = combineReducers({
    joypad: joypadReducer
});

export default rootReducer;