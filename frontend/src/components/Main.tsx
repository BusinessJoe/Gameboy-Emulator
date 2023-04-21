import { useEffect, useState } from 'react';
import Screen from './Screen';
import Joypad from './Joypad';
import JoypadRemap from './JoypadRemap';
import Save from './Save';
import { useAppSelector } from '../hooks/redux';
import JoypadDisplay from './JoypadDisplay';
import './Main.css';
import useGameboy from '../hooks/useGameboy';

const Main = () => {
  const [romData, setRomData] = useState<Uint8Array | undefined>(undefined);
  const { gameboy, screen, saveRam } = useGameboy(romData);
  const pressed = useAppSelector(state => state.joypad.pressed);
  const released = useAppSelector(state => state.joypad.released);

  useEffect(() => {
    if (pressed !== undefined && gameboy !== undefined) {
      gameboy.press_key(pressed);
    }
  }, [pressed, gameboy])
  
  useEffect(() => {
    if (released !== undefined && gameboy !== undefined) {
      gameboy.release_key(released);
    }
  }, [released, gameboy])

  return (
      <div className='main'>
        <div>
          <Screen screen={screen} onRomUpload={setRomData} />
          <Save onSave={saveRam} />
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