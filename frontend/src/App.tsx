import React, { useEffect, useState } from 'react';
import './App.css';
import init, { GameBoyState } from 'gameboy_emulator';
import Screen from './Screen';
import RomUpload from './RomUpload';

function App() {
  const gameboyRef = React.useRef<GameBoyState | undefined>(undefined);
  const [screen, setScreen] = useState<Uint8Array | undefined>(undefined);
  const [hasRom, setHasRom] = useState<boolean>(false);

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
      requestRef.current = requestAnimationFrame(renderFrame);
    }
  }, [hasRom])

  return (
    <div className="App">
      <header className="App-header">
        <Screen screen={screen}/>
        <p>
          {gameboyRef.current &&
            "Gameboy"
          }
          {JSON.stringify(gameboyRef.current)}
        </p>
        <RomUpload onUpload={(array) => {
          gameboyRef.current?.load_rom_web(array);
          setHasRom(true);
        }}/>
      </header>
    </div>
  );
}

export default App;
