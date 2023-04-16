import React, { MutableRefObject, useContext, useEffect, useState } from 'react';
import './App.css';
import Screen from './components/Screen';
import RomUpload from './components/RomUpload';
import Joypad from './components/Joypad';
import JoypadRemap from './components/JoypadRemap';
import { save_ram, load_ram } from './utils/database';
import GameboyContext, { GameboyProvider } from './components/GameboyContext';
import Main from './components/Main';
import { Provider } from 'react-redux';
import store from './store';

function App() {
  return (
    <div className="App">
      <header className="App-header">
      </header>
      <Provider store={store}>
        <GameboyProvider>
          <Main />
        </GameboyProvider>
      </Provider>
    </div>
  );
}

export default App;
