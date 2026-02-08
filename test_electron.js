const { app } = require('electron');
console.log('App object:', app);
if (app) {
    console.log('App path:', app.getPath('userData'));
    app.quit();
} else {
    console.error('App is undefined!');
    process.exit(1);
}
