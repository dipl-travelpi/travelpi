// V2 server (runs on the Pi): toggles the kill switch every 3–8 s and offers the status via
// REST (/api/status), Server-Sent Events (/sse) and WebSocket (/ws).
// Clients confirm every change they see with POST /ack (the same way for every method, so the
// confirmations cost the same). The server measures the delay from the change to the confirmation
// itself, so no clock sync between devices is needed.
//
// Usage: node tests/v2-server.js [port]
import { createServer } from 'node:http';
import { WebSocketServer } from 'ws';
import { getStatus } from '../src/lib/status.js';

const port = Number(process.argv[2] ?? 3001);

let version = 0;
let killSwitch = true;
let ignoreUpTo = 0; // changes from before the last reset are not counted
const changedAt = new Map();
let delays = [];
const sseClients = new Set();

const payload = () => JSON.stringify({ ...getStatus(), killSwitch, version });

function ack(v) {
	if (v > ignoreUpTo && changedAt.has(v)) delays.push(performance.now() - changedAt.get(v));
}

function change() {
	version++;
	killSwitch = !killSwitch;
	changedAt.set(version, performance.now());
	const data = payload();
	for (const res of sseClients) res.write(`data: ${data}\n\n`);
	for (const ws of wss.clients) ws.send(data);
	setTimeout(change, 3000 + Math.random() * 5000);
}

const server = createServer((req, res) => {
	if (req.url === '/api/status') {
		res.setHeader('content-type', 'application/json');
		res.end(payload());
	} else if (req.url === '/sse') {
		res.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-store' });
		sseClients.add(res);
		req.on('close', () => sseClients.delete(res));
	} else if (req.url === '/ack' && req.method === 'POST') {
		let body = '';
		req.on('data', (c) => (body += c));
		req.on('end', () => {
			ack(JSON.parse(body).version);
			res.end();
		});
	} else if (req.url === '/reset' && req.method === 'POST') {
		ignoreUpTo = version;
		delays = [];
		res.end();
	} else if (req.url === '/stats') {
		const cpu = process.cpuUsage();
		res.setHeader('content-type', 'application/json');
		res.end(JSON.stringify({ cpuMs: (cpu.user + cpu.system) / 1000, rssMb: process.memoryUsage().rss / 1e6, delays }));
	} else {
		res.writeHead(404).end();
	}
});

const wss = new WebSocketServer({ server, path: '/ws' });

server.listen(port, () => console.log(`V2 server on port ${port}`));
change();
