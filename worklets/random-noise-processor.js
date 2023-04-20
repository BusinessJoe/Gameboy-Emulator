class RandomNoiseProcessor extends AudioWorkletProcessor {
    constructor(...args) {
        super(...args);
        this.port.onMessage = (e) => {
            console.log('message payload:', e.data);
        };
    }
    
    process(inputs, outputs, parameters) {
        const output = outputs[0];
        output.forEach((channel) => {
            for (let i = 0; i < channel.length; i++) {
                channel[i] = (Math.random() * 2 - 1) * 0.5;
            }
        });
        return true;
    }
}

registerProcessor("random-noise-processor", RandomNoiseProcessor);