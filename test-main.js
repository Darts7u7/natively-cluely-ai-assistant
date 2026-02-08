console.log('ELECTRON_RUN_AS_NODE:', process.env.ELECTRON_RUN_AS_NODE);
console.log('NODE_PATH:', process.env.NODE_PATH);
const electron = require('electron');
console.log('Require electron type:', typeof electron);
if (typeof electron === 'string') {
    console.log('It is a string:', electron);
} else {
    console.log('Keys:', Object.keys(electron));
}

try {
    const { app } = require('electron');
    console.log('App object:', app);
    if (app) {
        app.on('ready', () => {
            console.log('Electron app is ready!');
            app.quit();
        });
    }
} catch (e) {
    console.error('Error requiring electron:', e);
}
