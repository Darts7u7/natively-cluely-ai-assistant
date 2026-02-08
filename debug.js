const electron = require('electron');
console.log('process.versions:', process.versions);
console.log('Type of electron:', typeof electron);

if (process.versions.electron) {
    console.log('Running in Electron version:', process.versions.electron);
} else {
    console.log('Running in Node.js (NOT Electron)');
}

