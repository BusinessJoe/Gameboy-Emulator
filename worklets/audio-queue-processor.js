class AudioQueueProcessor extends AudioWorkletProcessor {
    current_buffer;
    current_buffer_idx;
    audio_queue;
    freq;

    constructor(...args) {
        super(...args);
        this.current_buffer = null;
        this.current_buffer_idx = 0;
        this.audio_queue = [];
        this.freq = 0;

        this.port.onmessage = (e) => {
            if (this.audio_queue.length < 60) {
                this.audio_queue.push(e.data.payload);
            }
        };
    }
    
    process(inputs, outputs, parameters) {
        const output = outputs[0];
        const channel = output[0];
        for (let i = 0; i < channel.length; i++) {
            if (!this.current_buffer && this.audio_queue.length > 0) {
                this.current_buffer = this.audio_queue.shift();
            }

            if (!this.current_buffer) {
                channel[i] = 0;
            } else {
                //this.port.postMessage({'aaaa': this.audio_queue[0], 'b': this.idx, 'c': this.audio_queue[0][this.idx]});
                const sample = this.current_buffer[this.current_buffer_idx];
                channel[i] = sample;
                this.current_buffer_idx += 1;
                
                if (this.current_buffer_idx >= this.current_buffer.length) {
                    this.current_buffer = null;
                    this.current_buffer_idx = 0;
                }
            }
        }
            
        return true;
    }
}

registerProcessor("audio-queue-processor", AudioQueueProcessor);