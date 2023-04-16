import classNames from "classnames";
import React, { MouseEventHandler, useState } from "react";

function CircleButton(props: {
  onPress: () => void,
  onRelease: () => void,
  pressed: boolean,
}) {
  const [hovered, setHovered] = useState(false);
  
  const handleDown: MouseEventHandler<SVGElement> = (e) => {
    console.log(e);
    props.onPress();
  }

  const handleUp: MouseEventHandler<SVGElement> = (e) => {
    console.log(e);
    props.onRelease();
  }

  const cls = classNames({
    'button-hovered': hovered && !props.pressed,
    'button-pressed': props.pressed
  }); 
  
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="100%"
      version="1.1"
      viewBox="0 0 34.98 34.98"
      className={cls}
    >
      <g transform="translate(-18.641 -7.35)" 
        onPointerEnter={() => setHovered(true)} 
        onPointerLeave={() => {
          setHovered(false);
          if (props.pressed) {
            props.onRelease();
          }
        }} 
        onPointerDown={handleDown} 
        onPointerUp={handleUp}>
        <circle
          cx="36.131"
          cy="24.84"
          r="15.807"
          fill="currentColor"
          fillOpacity="1"
          stroke="#000"
          strokeOpacity="0"
          strokeWidth="3.365"
        ></circle>
      </g>
    </svg>
  );
}

export default CircleButton;