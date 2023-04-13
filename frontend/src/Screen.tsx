import { RefObject, useRef } from "react";
import './Screen.css';

const Screen = (props: {
    screen: Uint8Array | undefined
    focusRef: RefObject<HTMLDivElement>
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const canvas = canvasRef.current;
    const context = canvas?.getContext('2d');
    
    if (props.screen && context) {
        // allocate space for 4 color values (rgba) per screen pixel
        const arr = new Uint8ClampedArray(160 * 144 * 4);

        let color;
        for (let i = 0; i*4 < arr.length; i++) {
            switch (props.screen[i]) {
                case 0:
                    color = [255, 255, 255];
                    break;
                case 1:
                    color = [200, 200, 200];
                    break;
                case 2:
                    color = [100, 100, 100];
                    break;
                case 3:
                    color = [0, 0, 0];
                    break;
                default:
                    color = [255, 0, 0];
                    break;
            }

            arr[4*i + 0] = color[0];
            arr[4*i + 1] = color[1];
            arr[4*i + 2] = color[2];
            arr[4*i + 3] = 255;
        }

        const img_data = new ImageData(arr, 160, 144);
        context.putImageData(img_data, 0, 0);
    }

    let child;
    if (props.screen !== undefined) {
        child = (
            <canvas id="screen" width={160} height={144} ref={canvasRef}>Screen</canvas>
        );
    } else {
        child = <div>No Screen</div>;
    }

    return (
        <div id="screen-wrapper" ref={props.focusRef} tabIndex={0}>
            {child}
        </div>
    )
};

export default Screen;