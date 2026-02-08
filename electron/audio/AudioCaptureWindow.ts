import { BrowserWindow, ipcMain, desktopCapturer, app } from 'electron';
import * as path from 'path';
import { EventEmitter } from 'events';

export class AudioCaptureWindow extends EventEmitter {
    private window: BrowserWindow | null = null;
    private isRecording = false;

    constructor() {
        super();
        this.createWindow();
    }

    private createWindow() {
        if (this.window) return;

        this.window = new BrowserWindow({
            show: false, // Hidden window
            width: 100,
            height: 100,
            webPreferences: {
                nodeIntegration: false,
                contextIsolation: true,
                backgroundThrottling: false, // Keep running in background
                preload: path.join(__dirname, '../audio-capture/preload.js') // Points to dist-electron/audio-capture/preload.js
            }
        });

        // In dev mode, we point to the source HTML file
        // In prod, this might need adjustment if using asar
        const htmlPath = app.isPackaged
            ? path.join(process.resourcesPath, 'electron/audio-capture/index.html') // simplified guess for prod
            : path.join(process.cwd(), 'electron/audio-capture/index.html');

        this.window.loadFile(htmlPath).catch(err => {
            console.error('[AudioCaptureWindow] Failed to load HTML:', err);
        });

        // IPC Listeners
        ipcMain.on('AUDIO_DATA', (event, chunk) => {
            if (event.sender.id === this.window?.webContents.id) {
                this.emit('data', chunk);
            }
        });

        ipcMain.on('CAPTURE_ERROR', (event, message) => {
            if (event.sender.id === this.window?.webContents.id) {
                console.error('[AudioCaptureWindow] Renderer Error:', message);
                this.emit('error', new Error(message));
            }
        });

        this.window.on('closed', () => {
            this.window = null;
            this.isRecording = false;
        });
    }

    public async start(): Promise<void> {
        if (this.isRecording) return;

        if (!this.window) {
            this.createWindow();
        }

        try {
            console.log('[AudioCaptureWindow] Getting desktop sources...');
            // Find screen source
            const sources = await desktopCapturer.getSources({ types: ['screen'] });

            // Prefer "Entire Screen" or first screen
            // On Windows, usually "Screen 1" or "Entire Screen"
            const source = sources[0];

            if (!source) {
                throw new Error('No screen source found for audio capture');
            }

            console.log(`[AudioCaptureWindow] Using source: ${source.name} (${source.id})`);

            this.window?.webContents.send('START_CAPTURE', source.id);
            this.isRecording = true;
            this.emit('start');

        } catch (error) {
            console.error('[AudioCaptureWindow] Start failed:', error);
            this.emit('error', error);
        }
    }

    public stop(): void {
        if (!this.isRecording) return;

        this.window?.webContents.send('STOP_CAPTURE');
        this.isRecording = false;
        this.emit('stop');
    }

    public destroy(): void {
        this.stop();
        if (this.window) {
            this.window.destroy();
            this.window = null;
        }
    }
}
