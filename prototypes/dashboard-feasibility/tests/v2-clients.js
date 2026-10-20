// V2 clients (run on the Mac): for one method and 1, 5 and 10 clients, connect to the V2 server
// for 2 minutes each and confirm every status change (criteria B and E).
// Restart tests/v2-server.js on the Pi before each method so memory values are comparable.
//
// Usage: node tests/v2-clients.js <server-url> <polling-1s|polling-5s|sse|websocket>
// Example: node tests/v2-clients.js http://pi.local:3001 sse
import { writeFileSync, mkdirSync } from 'node:fs';

const [url, method] = process.argv.slice(2);
const DURATION_MS = Number(process.env.DURATION_MS ?? 120000);
const CLIENT_COUNTS = [1, 5, 10];

const ack = (version) => fetch(`${url}/ack`, { method: 'POST', body: JSON.stringify({ version }) });

function pollingClient(intervalMs) {
	return () => {
		let seen = -1;
		const timer = setInterval(async () => {
			const { version } = await (await fetch(`${url}/api/status`)).json();
			// confirm every change since the last poll, including ones that were skipped
			if (seen !== -1) for (let v = seen + 1; v <= version; v++) ack(v);
			seen = version;
		}, intervalMs);
		return () => clearInterval(timer);
	};
}

function sseClient() {
	const controller = new AbortController();
	(async () => {
		const res = await fetch(`${url}/sse`, { signal: controller.signal });
		const decoder = new TextDecoder();
		let buffer = '';
		for await (const chunk of res.body) {
			buffer += decoder.decode(chunk, { stream: true });
			let end;
			while ((end = buffer.indexOf('\n\n')) >= 0) {
				const event = buffer.slice(0, end);
				buffer = buffer.slice(end + 2);
				if (event.startsWith('data: ')) ack(JSON.parse(event.slice(6)).version);
			}
		}
	})().catch(() => {}); // aborted at the end of the run
	return () => controller.abort();
}

function webSocketClient() {
	const ws = new WebSocket(url.replace('http', 'ws') + '/ws');
	ws.onmessage = (e) => ack(JSON.parse(e.data).version);
	return () => ws.close();
}

const METHODS = {
	'polling-1s': pollingClient(1000),
	'polling-5s': pollingClient(5000),
	sse: sseClient,
	websocket: webSocketClient
};

const startClient = METHODS[method];
if (!startClient) {
	console.error(`Unknown method. Use one of: ${Object.keys(METHODS).join(', ')}`);
	process.exit(1);
}

const percentile = (values, p) => {
	const s = [...values].sort((a, b) => a - b);
	return s.length ? s[Math.min(s.length - 1, Math.floor((s.length * p) / 100))] : null;
};
const stats = async () => (await fetch(`${url}/stats`)).json();

const results = [];
for (const clients of CLIENT_COUNTS) {
	const stops = Array.from({ length: clients }, () => startClient());
	await new Promise((r) => setTimeout(r, 2000)); // let all clients connect
	await fetch(`${url}/reset`, { method: 'POST' });
	const before = await stats();
	await new Promise((r) => setTimeout(r, DURATION_MS));
	const after = await stats();
	stops.forEach((stop) => stop());

	const row = {
		method,
		clients,
		confirmations: after.delays.length,
		delayMedianMs: percentile(after.delays, 50),
		delayP95Ms: percentile(after.delays, 95),
		delayMaxMs: after.delays.length ? Math.max(...after.delays) : null,
		serverCpuPercent: ((after.cpuMs - before.cpuMs) / DURATION_MS) * 100,
		serverRssMb: after.rssMb
	};
	results.push(row);
	console.log(row);
	await new Promise((r) => setTimeout(r, 1000));
}

mkdirSync('results', { recursive: true });
writeFileSync(`results/v2-${method}.json`, JSON.stringify({ method, url, date: new Date().toISOString(), durationMs: DURATION_MS, results }, null, 2));
