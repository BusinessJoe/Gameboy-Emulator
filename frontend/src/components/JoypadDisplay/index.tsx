import DPadButton from './DPadButton';
import CircleButton from './CircleButton';
import PillButton from './PillButton';
import './JoypadDisplay.css';
import { useAppDispatch, useAppSelector } from '../../hooks/redux';
import { JoypadInput, inputPressed, inputReleased } from '../../reducers/joypadReducer';

const JoypadDisplay = () => {
    const current = useAppSelector((state) => state.joypad.current);
    const dispatch = useAppDispatch();

    const handlePress = (input: JoypadInput) => {
        dispatch(inputPressed(input));
    }

    const handleRealase = (input: JoypadInput) => {
        dispatch(inputReleased(input));
    }

    return (
        <div className="joypad-display">
            <div className="dpad">
                <DPadButton 
                    id="dpad-up"
                    pressed={current.has(JoypadInput.Up)} 
                    onPress={() => handlePress(JoypadInput.Up)}
                    onRelease={() => handleRealase(JoypadInput.Up)}
                />
                <DPadButton 
                    id="dpad-right"
                    pressed={current.has(JoypadInput.Right)} 
                    onPress={() => handlePress(JoypadInput.Right)}
                    onRelease={() => handleRealase(JoypadInput.Right)}
                />
                <DPadButton 
                    id="dpad-down"
                    pressed={current.has(JoypadInput.Down)} 
                    onPress={() => handlePress(JoypadInput.Down)}
                    onRelease={() => handleRealase(JoypadInput.Down)}
                />
                <DPadButton 
                    id="dpad-left"
                    pressed={current.has(JoypadInput.Left)} 
                    onPress={() => handlePress(JoypadInput.Left)}
                    onRelease={() => handleRealase(JoypadInput.Left)}
                />
            </div>
            <div className="circle-button-wrapper">
                <div className="circle-button" id="a-button">
                    <CircleButton 
                        pressed={current.has(JoypadInput.A)} 
                        onPress={() => handlePress(JoypadInput.A)}
                        onRelease={() => handleRealase(JoypadInput.A)}
                    />
                </div>
                <div className="circle-button" id="b-button">
                    <CircleButton 
                        pressed={current.has(JoypadInput.B)} 
                        onPress={() => handlePress(JoypadInput.B)}
                        onRelease={() => handleRealase(JoypadInput.B)}
                    />
                </div>
            </div>
            <div className="pill-button-wrapper">
                <div className="pill-button" id="select-button">
                    <PillButton 
                        pressed={current.has(JoypadInput.Select)} 
                        onPress={() => handlePress(JoypadInput.Select)}
                        onRelease={() => handleRealase(JoypadInput.Select)}
                    />
                </div>
                <div className="pill-button" id="start-button">
                    <PillButton 
                        pressed={current.has(JoypadInput.Start)} 
                        onPress={() => handlePress(JoypadInput.Start)}
                        onRelease={() => handleRealase(JoypadInput.Start)}
                    />
                </div>
            </div>
        </div>
    )
}

export default JoypadDisplay;