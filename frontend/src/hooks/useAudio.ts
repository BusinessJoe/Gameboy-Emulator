import { useRef, useCallback } from "react";

const useAudio = () => {
    const audioContext = useRef<AudioContext|null>(null);
    const node = useRef<AudioWorkletNode|null>(null);

    const queueAudio = useCallback((audio: Float32Array) => {
        node.current?.port.postMessage({ type: "queue", payload: audio });
    }, []);

    const init = useCallback(async () => {
        console.log('audio init');
        audioContext.current = new AudioContext({
            sampleRate: 44100
        });
        try {
            await audioContext.current.audioWorklet.addModule(process.env.PUBLIC_URL + "/worklets/audio-queue-processor.js");
            node.current = new AudioWorkletNode(
                audioContext.current,
                "audio-queue-processor"
            );
            node.current.connect(audioContext.current.destination);
            node.current.port.onmessage = (e) => console.log(e);
            node.current.port.onmessageerror = (e) => console.error(e);
            node.current.onprocessorerror = (e) => console.error(e);
            console.log('initialized audio');
        } catch (e) {
            console.log(e);
        }
    }, []);

    const resume = useCallback(async () => {
        console.log('audio resume');
        await audioContext.current?.resume();
    }, []);

    return { init, resume, queueAudio };
}

export default useAudio