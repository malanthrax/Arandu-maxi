const fs = require('fs');
const path = require('path');
const { exec, spawn } = require('child_process');

const args = process.argv.slice(2);
const command = args[0] || 'all';

// Resolve project root from the script location: Arandu-maxi/scripts/../
const projectRoot = path.resolve(__dirname, '..');

const FILES_TO_CHECK = [
    path.join('backend', 'src', 'lib.rs'),
    path.join('backend', 'src', 'main.rs'),
    path.join('backend', 'Cargo.toml'),
    path.join('frontend', 'index.html'),
    path.join('frontend', 'llama-custom', 'index.html'),
    path.join('backend', 'tauri.conf.json')
];

function verifyFiles() {
    console.log('Verifying required files...');
    let allExist = true;
    for (const file of FILES_TO_CHECK) {
        const fullPath = path.join(projectRoot, file);
        if (fs.existsSync(fullPath)) {
            console.log(`[OK] ${file}`);
        } else {
            console.error(`[MISSING] ${file}`);
            allExist = false;
        }
    }
    return allExist;
}

function checkBuild() {
    console.log('Running cargo check in backend...');
    const backendPath = path.join(projectRoot, 'backend');
    
    return new Promise((resolve, reject) => {
        const child = spawn('cargo', ['check'], {
            cwd: backendPath,
            stdio: 'inherit',
            shell: true
        });

        child.on('close', (code) => {
            if (code === 0) {
                console.log('✅ Build check passed.');
                resolve();
            } else {
                console.error(`❌ cargo check failed with code ${code}`);
                reject(new Error(`Exit code ${code}`));
            }
        });

        child.on('error', (err) => {
            console.error(`❌ Failed to start process: ${err.message}`);
            reject(err);
        });
    });
}

async function run() {
    try {
        if (command === 'verify-files' || command === 'all') {
            if (!verifyFiles()) {
                console.error('File verification failed.');
                process.exit(1);
            }
        }

        if (command === 'check-build' || command === 'all') {
            await checkBuild();
        }
    } catch (err) {
        console.error('Operation failed:', err);
        process.exit(1);
    }
}

run();
