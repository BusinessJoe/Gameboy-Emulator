import { useEffect, useState } from "react";

export function useKeyDown() {
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
}