import React, { useCallback, useContext, useEffect, useState } from 'react';
import Screen from './Screen';
import RomUpload from './RomUpload';
import Joypad from './Joypad';
import JoypadRemap from './JoypadRemap';
import { load_ram } from '../utils/database';
import GameboyContext from './GameboyContext';
import Save from './Save';
import { useAppSelector } from '../hooks/redux';
import JoypadDisplay from './JoypadDisplay';
import './Main.css';
import useAudio from '../hooks/useAudio';

const Main = () => {
  const { gameboy } = useContext(GameboyContext);
  const pressed = useAppSelector(state => state.joypad.pressed);
  const released = useAppSelector(state => state.joypad.released);
  const [screen, setScreen] = useState<Uint8Array | undefined>(undefined);
  const [hasRom, setHasRom] = useState<boolean>(false);

  // https://css-tricks.com/using-requestanimationframe-with-react-hooks/
  // Use useRef for mutable variables that we want to persist
  // without triggering a re-render on their change
  const requestRef = React.useRef<number>();
  const nextTimeRef = React.useRef<number>(0);

  const { init, resume, queueAudio } = useAudio();


  const renderFrame = useCallback((time: number) => {
    if (nextTimeRef.current === 0) {
      nextTimeRef.current = time + 1000 / 60;
    }

    if (time >= nextTimeRef.current) {
      setScreen(gameboy.get_web_screen());
      gameboy.tick_for_frame();
      queueAudio(gameboy.get_queued_audio());
      // let array = [];
      // for (let i = 0; i < 735; i++) {
      //   array.push(0.1 * (Math.random()*2-1));
      // }
      // audioHook.queueAudio(Float32Array.from(array));

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
  }, [gameboy, queueAudio]);

  useEffect(() => {
    setScreen(gameboy.get_web_screen());
    init();
  }, [gameboy, init]);

  useEffect(() => {
    if (hasRom) {
      requestRef.current = requestAnimationFrame(renderFrame);
    }
  }, [hasRom, renderFrame])

  useEffect(() => {
    if (pressed !== undefined) {
      gameboy.press_key(pressed);
    }
  }, [pressed, gameboy])
  
  useEffect(() => {
    if (released !== undefined) {
      gameboy.release_key(released);
    }
  }, [released, gameboy])

  const handleRomUpload = useCallback((array: Uint8Array) => {
    gameboy.load_rom_web(array);
    const identifier = gameboy.game_name();

    if (identifier) {
      load_ram(identifier)
        .then(ram => {
          console.log('save found for game ', identifier, ram);
          if (gameboy.load_save(ram)) {
            console.log('loaded save');
          } else {
            console.log('failed to load save');
          }
          setHasRom(true);
        })
        .catch(() => {
          // no save found
          console.log('no save found for game ', identifier);
          setHasRom(true);
        })
    } else {
      setHasRom(true);
    }
    resume();
  }, [gameboy, resume]);

  return (
      <div className='main'>
        <div>
          <Screen screen={screen} />
          <RomUpload onUpload={handleRomUpload} />
          <Save />
        </div>
        <div className='sidebar'>
          <JoypadDisplay />
          <Joypad>
            <JoypadRemap />
          </Joypad>
        </div>
      </div>
  );
}

export default Main;