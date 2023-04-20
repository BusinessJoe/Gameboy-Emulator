import classNames from "classnames";
import React, { MouseEventHandler, useState } from "react";

function PillButton(props: {
  onPress: () => void,
  onRelease: () => void,
  pressed: boolean,
}) {
  const [hovered, setHovered] = useState(false);
  
  const handleDown: MouseEventHandler<SVGElement> = (e) => {
    props.onPress();
  }

  const handleUp: MouseEventHandler<SVGElement> = (e) => {
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
      viewBox="0 0 65.776 22.164"
      className={cls}
    >
      <g transform="translate(-109.217 -19.896)"
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
          fill="currentColor"
          fillOpacity="1"
          stroke="#000"
          strokeOpacity="0"
          strokeWidth="4.569"
          d="M119.97 22.18h44.272a8.467 8.467 45 018.466 8.467v.662a8.467 8.467 135 01-8.466 8.467h-44.275a8.465 8.465 45.005 01-8.465-8.467v-.662a8.468 8.468 135.005 018.468-8.467z"
        ></path>
      </g>
    </svg>
  );
}

export default PillButton;