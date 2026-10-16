#!/usr/bin/env node

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { exec } from 'node:child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DIST_DIR = path.resolve(__dirname, '../dist');
const PROJECT_DIR = process.cwd();
const ORCRUSTRATOR_DIR = path.join(PROJECT_DIR, 'orcrustrator');

// Ensure dist directory exists
if (!fs.existsSync(DIST_DIR)) {
  console.error('\nError: UI dist directory not found.');
  console.error('Please run "npm run build" inside the package before launching.\n');
  process.exit(1);
}

// MIME types for static assets
const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.svg': 'image/svg+xml',
  '.ico': 'image/x-icon',
  '.woff2': 'font/woff2',
};

// Open URL in default browser cross-platform
function openBrowser(url) {
  const startCmd =
    process.platform === 'darwin'
      ? `open "${url}"`
      : process.platform === 'win32'
      ? `start "" "${url}"`
      : `xdg-open "${url}"`;

  exec(startCmd, (err) => {
    if (err) {
      // browser open failure is non-fatal
    }
  });
}

// Create HTTP server
const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  const pathname = url.pathname;

  // Set CORS headers for local development
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(204);
    res.end();
    return;
  }

  // API Route: Get project workspace status & manifests
  if (pathname === '/api/workspace' && req.method === 'GET') {
    const defPath = path.join(ORCRUSTRATOR_DIR, 'definitions.toml');
    const ovrPath = path.join(ORCRUSTRATOR_DIR, 'overrides.toml');

    const definitions = fs.existsSync(defPath) ? fs.readFileSync(defPath, 'utf8') : null;
    const overrides = fs.existsSync(ovrPath) ? fs.readFileSync(ovrPath, 'utf8') : null;

    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(
      JSON.stringify({
        projectDir: PROJECT_DIR,
        hasDefinitions: definitions !== null,
        hasOverrides: overrides !== null,
        definitions,
        overrides,
      })
    );
    return;
  }

  // API Route: Save overrides.toml
  if (pathname === '/api/overrides' && req.method === 'POST') {
    let body = '';
    req.on('data', (chunk) => {
      body += chunk;
    });
    req.on('end', () => {
      try {
        const data = JSON.parse(body);
        if (!fs.existsSync(ORCRUSTRATOR_DIR)) {
          fs.mkdirSync(ORCRUSTRATOR_DIR, { recursive: true });
        }
        const ovrPath = path.join(ORCRUSTRATOR_DIR, 'overrides.toml');
        fs.writeFileSync(ovrPath, data.content, 'utf8');

        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: true, path: ovrPath }));
      } catch (err) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ success: false, error: err.message }));
      }
    });
    return;
  }

  // API Route: Run orcrustrator compile
  if (pathname === '/api/compile' && req.method === 'POST') {
    exec('orcrustrator compile --diff', { cwd: PROJECT_DIR }, (err, stdout, stderr) => {
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(
        JSON.stringify({
          success: !err,
          stdout: stdout || '',
          stderr: stderr || '',
        })
      );
    });
    return;
  }

  // Static File Serving
  let relativePath = pathname === '/' ? 'index.html' : pathname.replace(/^\//, '');
  let filePath = path.join(DIST_DIR, relativePath);

  // Security: Prevent directory traversal outside DIST_DIR
  if (!filePath.startsWith(DIST_DIR)) {
    res.writeHead(403, { 'Content-Type': 'text/plain' });
    res.end('Forbidden');
    return;
  }

  // Fallback to index.html for SPA client-side routes
  if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
    filePath = path.join(DIST_DIR, 'index.html');
  }

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' });
      res.end('Not Found');
      return;
    }

    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';

    res.writeHead(200, { 'Content-Type': contentType });
    res.end(data);
  });
});

// Port selection and startup
const DEFAULT_PORT = parseInt(process.env.PORT, 10) || 4173;

function startServer(port) {
  server.listen(port, () => {
    const url = `http://localhost:${port}`;
    console.log('\n============================================================');
    console.log('   ORCRUSTRATOR STUDIO (NPX)');
    console.log('============================================================');
    console.log(`  Local UI URL : ${url}`);
    console.log(`  Project Path : ${PROJECT_DIR}`);
    console.log('  Mode         : Decoupled Workspace Bridge (Zero Engine Bloat)');
    console.log('============================================================\n');
    console.log('Opening browser... Press Ctrl+C to stop.\n');
    openBrowser(url);
  });

  server.on('error', (err) => {
    if (err.code === 'EADDRINUSE') {
      console.log(`Port ${port} in use, trying port ${port + 1}...`);
      startServer(port + 1);
    } else {
      console.error('Server error:', err);
      process.exit(1);
    }
  });
}

startServer(DEFAULT_PORT);
