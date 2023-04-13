import React, { MutableRefObject, useEffect, useState } from 'react';
import './App.css';
import init, { GameBoyState } from 'gameboy_emulator';
import Screen from './Screen';
import RomUpload from './RomUpload';
import Joypad from './Joypad';

function App() {
  const gameboyRef = React.useRef<GameBoyState | undefined>(undefined);
  const [screen, setScreen] = useState<Uint8Array | undefined>(undefined);
  const [hasRom, setHasRom] = useState<boolean>(false);

  const screenRef = React.useRef<HTMLDivElement>(null);

  // https://css-tricks.com/using-requestanimationframe-with-react-hooks/
  // Use useRef for mutable variables that we want to persist
  // without triggering a re-render on their change
  const requestRef = React.useRef<number>();
  const nextTimeRef = React.useRef<number>(0);
  const frameCountRef = React.useRef<number>(0);

  const renderFrame = (time: number) => {
    if (nextTimeRef.current === 0) {
      nextTimeRef.current = time + 1000 / 60;
    }

    if (time >= nextTimeRef.current) {
        setScreen(gameboyRef.current?.get_web_screen());
        gameboyRef.current?.tick_for_frame();
        
        frameCountRef.current += 1;
        // if (frameCountRef.current % (5 * 60) === 0) {
        //     save(gameboy);
        // }
        nextTimeRef.current += 1000 / 60;

        if (nextTimeRef.current <= time) {
          console.warn("frame took too long");
          nextTimeRef.current = time + 1000 / 60;
        }
    }
    requestRef.current = requestAnimationFrame(renderFrame);
  }

  useEffect(() => {
    init().then(() => {
      gameboyRef.current = GameBoyState.new();
      setScreen(gameboyRef.current?.get_web_screen());
    })
  }, []);

  useEffect(() => {
    if (hasRom) {
      screenRef.current?.focus();
      requestRef.current = requestAnimationFrame(renderFrame);
    }
  }, [hasRom])

  return (
    <div className="App">
      <header className="App-header">
        {screenRef.current && 
          <Joypad focusRef={screenRef} onJoypadInput={(key, down) => {
            if (down) {
              gameboyRef.current?.press_key(key);
            } else {
              gameboyRef.current?.release_key(key);
            }
          }} />
        }
        <Screen screen={screen} focusRef={screenRef} />
        <RomUpload onUpload={(array) => {
          gameboyRef.current?.load_rom_web(array);
          setHasRom(true);
        }}/>
      </header>
      <div>
        <h1>Keybinds</h1>
        <table>
          <tr>
            <td>A</td>
            <td>B</td>
            <td>Start</td>
            <td>Select</td>
            <td>Directions</td>
          </tr>
          <tr>
            <td>1</td>
            <td>2</td>
            <td>3</td>
            <td>4</td>
            <td>Arrow Keys</td>
          </tr>
        </table>
      </div>
    </div>
  );
}

export default App;
