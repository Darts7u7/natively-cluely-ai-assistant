// This script runs in the hidden renderer process
// It uses the exposed API from preload.ts

// Treat as module to avoid global scope issues
export { };

interface AudioCaptureAPI {
    onStartCapture: (callback: (sourceId: string) => void) => void;
    onStopCapture: (callback: () => void) => void;
    sendAudioData: (chunk: Uint8Array) => void;
    sendError: (message: string) => void;
}

// Access the exposed API via window
const api = (window as any).audioCapture as AudioCaptureAPI;

let audioContext: AudioContext | null = null;
let mediaStream: MediaStream | null = null;
let processor: ScriptProcessorNode | null = null;
let source: MediaStreamAudioSourceNode | null = null;

const TARGET_SAMPLE_RATE = 16000;

api.onStartCapture(async (sourceId: string) => {
    try {
        console.log('[AudioCapture] Starting capture for source:', sourceId);

        // Stop any existing capture
        stopCapture();

        // 1. Get the stream
        const stream = await navigator.mediaDevices.getUserMedia({
            audio: {
                mandatory: {
                    chromeMediaSource: 'desktop',
                    chromeMediaSourceId: sourceId
                }
            } as any,
            video: {
                mandatory: {
                    chromeMediaSource: 'desktop',
                    chromeMediaSourceId: sourceId,
                    maxWidth: 1,
                    maxHeight: 1
                }
            } as any // We need video constraint to satisfy desktopCapturer, but we don't use it
        });

        mediaStream = stream;

        // 2. Create AudioContext at 16kHz
        // Pass latencyHint: 'interactive' for low latency
        audioContext = new AudioContext({
            sampleRate: TARGET_SAMPLE_RATE,
            latencyHint: 'interactive'
        });

        // 3. Create Source
        source = audioContext.createMediaStreamSource(stream);

        // 4. Create Processor
        // Buffer size 4096 @ 16kHz ~= 256ms
        // We utilize a ScriptProcessorNode for simplicity in this fallback.
        // Ideally AudioWorklet, but ScriptProcessor is fine for 16kHz mono.
        processor = audioContext.createScriptProcessor(4096, 1, 1);

        processor.onaudioprocess = (e) => {
            const inputData = e.inputBuffer.getChannelData(0);

            // Convert Float32 to Int16
            const pcmData = new Int16Array(inputData.length);
            for (let i = 0; i < inputData.length; i++) {
                // Clamp between -1 and 1
                let s = Math.max(-1, Math.min(1, inputData[i]));
                // Scale to Int16 range
                s = s < 0 ? s * 0x8000 : s * 0x7FFF;
                pcmData[i] = s;
            }

            // Send as bytes
            const buffer = new Uint8Array(pcmData.buffer);
            api.sendAudioData(buffer);
        };

        // 5. Connect
        source.connect(processor);
        processor.connect(audioContext.destination); // Needed for processing to happen

        console.log('[AudioCapture] Capture started');

    } catch (err: any) {
        console.error('[AudioCapture] Failed to start:', err);
        api.sendError(err.message || String(err));
    }
});

api.onStopCapture(() => {
    console.log('[AudioCapture] Stopping capture');
    stopCapture();
});

function stopCapture() {
    if (processor) {
        processor.disconnect();
        processor.onaudioprocess = null;
        processor = null;
    }
    if (source) {
        source.disconnect();
        source = null;
    }
    if (mediaStream) {
        mediaStream.getTracks().forEach(track => track.stop());
        mediaStream = null;
    }
    if (audioContext) {
        audioContext.close();
        audioContext = null;
    }
}
