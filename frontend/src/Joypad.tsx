import React, { useEffect } from 'react';

const mapCodeToKey = (code: KeyboardEvent["code"]): number => {
    let key = -1;
    if (code === "Digit1") {
        key = 0;
    } else if (code === "Digit2") {
        key = 1;
    } else if (code === "Digit3") {
        key = 2;
    } else if (code === "Digit4") {
        key = 3;
    } else if (code === "ArrowLeft") {
        key = 4;
    } else if (code === "ArrowRight") {
        key = 5;
    } else if (code === "ArrowUp") {
        key = 6;
    } else if (code === "ArrowDown") {
        key = 7;
    }
    return key;
}

const Joypad = (props: {
    focusRef: React.RefObject<HTMLElement>,
    onJoypadInput: (key: number, down: boolean) => void
}) => {
    if (props.focusRef.current) {
        // set up keyboard listeners
        props.focusRef.current.onkeydown = (e: KeyboardEvent) => {
            e.preventDefault();
            const key = mapCodeToKey(e.code);

            if (key >= 0) {
                props.onJoypadInput(key, true);
            }
        }
        props.focusRef.current.onkeyup = (e: KeyboardEvent) => {
            e.preventDefault();
            const key = mapCodeToKey(e.code);

            if (key >= 0) {
                props.onJoypadInput(key, false);
            }
        }
    }

    return (null);
}

export default Joypad;