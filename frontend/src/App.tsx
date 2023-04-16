import './App.css';
import { GameboyProvider } from './components/GameboyContext';
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
