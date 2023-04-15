import React, { MutableRefObject, useContext, useEffect, useState } from 'react';
import './App.css';
import Screen from './components/Screen';
import RomUpload from './components/RomUpload';
import Joypad from './components/Joypad';
import JoypadRemap from './components/JoypadRemap';
import { save_ram, load_ram } from './utils/database';
import GameboyContext, { GameboyProvider } from './components/GameboyContext';
import Main from './components/Main';

function App() {
  return (
    <div className="App">
      <header className="App-header">
      </header>
      <GameboyProvider>
        <Main />
      </GameboyProvider>
    </div>
  );
}

export default App;
