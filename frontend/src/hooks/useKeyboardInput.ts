import { useEffect, useState } from "react";

const useKeyDown = () => {
  const [key, setKey] = useState('');
  useEffect(() => {
    function handleDown(e: KeyboardEvent) {
      setKey(e.key);
    }
    window.addEventListener('keydown', handleDown);
    return () => {
      window.removeEventListener('keydown', handleDown);
    };
  }, []);
  return { key };
};

export { useKeyDown }