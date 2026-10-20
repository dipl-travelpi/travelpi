// V1c: 5, 10 and 30 simulated devices load the dashboard over and over for 30 s each (criterion E).
// 30 devices is far above normal use and shows how much the Pi can serve at most.
// Each device repeats exactly the requests Chrome made for one page load in V1a,
// so run tests/v1-chrome.js for the same mode first.
//
// Usage: node tests/v1-load.js <server-url> <csr|ssr|ssg>
import { readFileSync, writeFileSync } from 'node:fs';
import autocannon from 'autocannon';

const [url, mode] = process.argv.slice(2);
const DEVICES = [5, 10, 30]; // 5 = minimum from the requirements (/LL070/)
const DURATION_S = 30;

const chrome = JSON.parse(readFileSync(`results/v1-chrome-${mode}.json`, 'utf8'));
const pageRequests = chrome.runs[1].requests.filter((p) => p !== '/stats');

const stats = async () => (await fetch(`${url}/stats`)).json();
const results = [];

for (const devices of DEVICES) {
	const before = await stats();
	const r = await autocannon({
		url,
		connections: devices,
		duration: DURATION_S,
		headers: { 'accept-encoding': 'gzip, deflate, br' }, // same as Chrome and Safari
		requests: pageRequests.map((path) => ({ method: 'GET', path }))
	});
	const after = await stats();

	const row = {
		devices,
		pageLoadsPerSecond: r.requests.average / pageRequests.length,
		requestLatencyMedianMs: r.latency.p50,
		requestLatencyP97_5Ms: r.latency.p97_5,
		serverCpuPercent: ((after.cpuMs - before.cpuMs) / (DURATION_S * 1000)) * 100,
		serverRssMb: after.rssMb,
		errors: r.errors + r.non2xx
	};
	results.push(row);
	console.log(row);
}

writeFileSync(
	`results/v1-load-${mode}.json`,
	JSON.stringify({ mode, url, date: new Date().toISOString(), durationS: DURATION_S, pageRequests, results }, null, 2)
);
