// Serves one build (csr, ssr or ssg) for the measurements.
// Usage: node server.js <csr|ssr|ssg> [port]
//
// - Caching is disabled so every page load is a first load.
// - Every response is gzip-compressed the same way, as a real web server would do.
// - GET /stats returns the CPU time and memory of this process.
import { createServer } from 'node:http';
import compression from 'compression';

const mode = process.argv[2];
if (!['csr', 'ssr', 'ssg'].includes(mode)) {
	console.error('Usage: node server.js <csr|ssr|ssg> [port]');
	process.exit(1);
}
const port = Number(process.argv[3] ?? 3000);
const { handler } = await import(`./build-${mode}/handler.js`);
const compress = compression();

createServer((req, res) => {
	if (req.url === '/stats') {
		const cpu = process.cpuUsage();
		res.setHeader('content-type', 'application/json');
		return res.end(JSON.stringify({ cpuMs: (cpu.user + cpu.system) / 1000, rssMb: process.memoryUsage().rss / 1e6 }));
	}

	delete req.headers['if-none-match'];
	delete req.headers['if-modified-since'];
	const setHeader = res.setHeader.bind(res);
	res.setHeader = (name, value) => {
		const n = name.toLowerCase();
		if (n === 'etag' || n === 'last-modified') return res;
		return setHeader(name, n === 'cache-control' ? 'no-store' : value);
	};
	setHeader('cache-control', 'no-store');

	compress(req, res, () => handler(req, res));
}).listen(port, () => console.log(`${mode.toUpperCase()} on port ${port}`));
