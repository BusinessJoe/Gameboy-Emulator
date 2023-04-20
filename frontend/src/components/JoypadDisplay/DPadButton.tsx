import React, { MouseEventHandler, useState } from "react";
import classNames from "classnames";

function DPadButton(props: {
    onPress: () => void,
    onRelease: () => void,
    pressed: boolean,
    id?: string,
}) {
  const [hovered, setHovered] = useState(false);

  const handleDown: MouseEventHandler<SVGElement> = (e) => {
    props.onPress();
  }

  const handleUp: MouseEventHandler<SVGElement> = (e) => {
    props.onRelease();
  }

  const cls = classNames({
    'dpad-button': true,
    'button-hovered': hovered && !props.pressed,
    'button-pressed': props.pressed,
  }); 

  return (
    <div 
    id={props.id}
    className={cls}>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="35%"
        version="1.1"
        viewBox="0 0 45.267 80.841"
        >
        <g id="layer1" transform="translate(-66.44 -72.89)" 
          onPointerEnter={() => setHovered(true)} 
          onPointerLeave={() => {
            setHovered(false);
            if (props.pressed) {
              props.onRelease();
            }
          }} 
          onPointerDown={handleDown} 
          onPointerUp={handleUp}>
          <path
            id="path3085"
            fill="currentColor"
            fillOpacity="1"
            stroke="#000"
            strokeOpacity="0"
            strokeWidth="3.365"
            d="M68.122 112.453V76.46c0-1.041.844-1.886 1.885-1.886h19.066v51.018h-6.664a2.172 2.172 0 01-1.491-.593l-12.465-11.776a1.058 1.058 0 01-.331-.769zm41.902 0V76.46a1.886 1.886 0 00-1.886-1.886H89.073v51.018h6.663c.555 0 1.089-.212 1.492-.593l12.464-11.776c.212-.2.332-.478.332-.769z"
            className="UnoptimicedTransforms"
          ></path>
        </g>
      </svg>
    </div>
  );
}

export default DPadButton;
