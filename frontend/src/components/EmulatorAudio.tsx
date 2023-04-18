import { useEffect, useRef, useState } from "react";

const EmulatorAudio = (props: {
    play: boolean,
}) => {
    const audioContext = useRef(new AudioContext());
    const node = useRef<AudioWorkletNode|null>(null);

    useEffect(() => {
        async function loadAudio() {
            try {
                await audioContext.current.audioWorklet.addModule(process.env.PUBLIC_URL + "/worklets/random-noise-processor.js");
                node.current = new AudioWorkletNode(
                    audioContext.current,
                    "random-noise-processor"
                );
                node.current.connect(audioContext.current.destination);
            } catch (e) {
                console.log(e);
            }
        }
        loadAudio();
    }, [])

    useEffect(() => {
        if (props.play) {
            console.log("playing audio");
            audioContext.current.resume();
        }
    }, [props.play])

    return null;
}

export default EmulatorAudio;