import { useRef, useCallback, useEffect } from "react";

const useAudio = (getNextFrameAudioCallback: () => Float32Array) => {
    const callbackRef = useRef<() => Float32Array>();
    const audioContext = useRef<AudioContext | undefined>(undefined);
    const node = useRef<AudioWorkletNode | undefined>(undefined);

    useEffect(() => {
        callbackRef.current = getNextFrameAudioCallback;
    }, [getNextFrameAudioCallback]);

    const init = useCallback(async () => {
        try {
            audioContext.current = new AudioContext({
                sampleRate: 44100
            });
            await audioContext.current.audioWorklet.addModule(process.env.PUBLIC_URL + "/worklets/audio-queue-processor.js");
            node.current = new AudioWorkletNode(
                audioContext.current,
                "audio-queue-processor"
            );
            node.current.connect(audioContext.current.destination);
            node.current.port.onmessage = (e) => {
                if (e.data.type === "get-audio") {
                    node.current?.port.postMessage({ type: "queue", payload: callbackRef.current?.() });
                }
            }
            node.current.port.onmessageerror = (e) => console.error(e);
            node.current.onprocessorerror = (e) => console.error(e);
            console.log('Initialized audio');
        } catch (e) {
            console.error('Failed to initialize audio:', e);
        }
    }, []);

    const resume = useCallback(async () => {
        if (audioContext.current === undefined) {
            throw new TypeError('Audio context is undefined');
        }
        await audioContext.current.resume();
        console.log('Resumed audio context');
    }, []);

    return { init, resume };
}

export default useAudio