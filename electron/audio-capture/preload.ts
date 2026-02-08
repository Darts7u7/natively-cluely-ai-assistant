import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('audioCapture', {
    onStartCapture: (callback: (sourceId: string) => void) => {
        ipcRenderer.on('START_CAPTURE', (_event, sourceId) => callback(sourceId));
    },
    onStopCapture: (callback: () => void) => {
        ipcRenderer.on('STOP_CAPTURE', () => callback());
    },
    sendAudioData: (chunk: Uint8Array) => {
        ipcRenderer.send('AUDIO_DATA', chunk);
    },
    sendError: (message: string) => {
        ipcRenderer.send('CAPTURE_ERROR', message);
    }
});
