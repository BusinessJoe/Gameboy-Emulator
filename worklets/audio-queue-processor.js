class AudioQueueProcessor extends AudioWorkletProcessor {
    current_buffer;
    current_buffer_idx;
    audio_queue;
    requested_frames;
    freq;
    last_sample;

    constructor(...args) {
        super(...args);
        this.current_buffer = null;
        this.current_buffer_idx = 0;
        this.audio_queue = [];
        this.requested_frames = 0;
        this.freq = 0;
        this.last_samples = [0, 0];

        this.port.onmessage = (e) => {
            if (e.data.type === 'queue') {
                if (this.requested_frames > 0) {
                    this.requested_frames -= 1;
                }
                if (this.audio_queue.length < 60) {
                    this.audio_queue.push(e.data.payload);
                }
            }
        };
    }
    
    process(inputs, outputs, parameters) {
        const output = outputs[0];
        const channels = [output[0], output[1]];
        if (channels[0].length !== channels[1].length) {
            throw new Error("Channel lengths do not match");
        }
        const length = channels[0].length;

        for (let i = 0; i < length; i++) {
            if (!this.current_buffer && this.audio_queue.length > 0) {
                this.current_buffer = this.audio_queue.shift();
            }

            if (!this.current_buffer) {
                channels[0][i] = this.last_samples[0];
                channels[1][i] = this.last_samples[1];
            } else {
                //this.port.postMessage({'aaaa': this.audio_queue[0], 'b': this.idx, 'c': this.audio_queue[0][this.idx]});
                channels[0][i] = this.current_buffer[this.current_buffer_idx];
                channels[1][i] = this.current_buffer[this.current_buffer_idx + 1];
                this.current_buffer_idx += 2;
                
                if (this.current_buffer_idx >= this.current_buffer.length) {
                    this.current_buffer = null;
                    this.current_buffer_idx = 0;
                }
            }
        }
        this.last_samples = [channels[0][length - 1], channels[1][length - 1]]

        if (this.audio_queue.length < 3 && this.requested_frames === 0) {
            this.port.postMessage({ type: 'get-audio' });
            this.requested_frames += 1;
        }
            
        return true;
    }
}

registerProcessor("audio-queue-processor", AudioQueueProcessor);