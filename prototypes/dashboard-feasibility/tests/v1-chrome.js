// V1a: load the dashboard 16 times in Chrome with mobile throttling and measure
// - load timings in the browser (criterion A)
// - CPU time the server spends per page load (criterion E)
// - which status time is contained in the delivered HTML (criterion B)
//
// Usage: node tests/v1-chrome.js <server-url> <csr|ssr|ssg>
// Example: node tests/v1-chrome.js http://pi.local:3000 ssr
import { writeFileSync, mkdirSync } from 'node:fs';
import puppeteer from 'puppeteer-core';

const [url, mode] = process.argv.slice(2);
const RUNS = 16; // run 1 is a warm-up and is not evaluated
const CHROME = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';

// Lighthouse mobile defaults ("Slow 4G", 4x CPU slowdown)
const NETWORK = { offline: false, latency: 150, downloadThroughput: 1.6e6 / 8, uploadThroughput: 750e3 / 8 };
const CPU_SLOWDOWN = 4;

const stats = async () => (await fetch(`${url}/stats`)).json();

const browser = await puppeteer.launch({ executablePath: CHROME, headless: true });
const runs = [];
let statsAfterWarmup;

for (let run = 1; run <= RUNS; run++) {
	// fresh incognito context: empty cache, no cookies
	const context = await browser.createBrowserContext();
	const page = await context.newPage();
	await page.setViewport({ width: 390, height: 844, isMobile: true, hasTouch: true });
	const cdp = await page.createCDPSession();
	await cdp.send('Network.enable');
	await cdp.send('Network.emulateNetworkConditions', NETWORK);
	await cdp.send('Emulation.setCPUThrottlingRate', { rate: CPU_SLOWDOWN });

	const requests = [];
	page.on('request', (r) => requests.push(new URL(r.url()).pathname));

	await page.goto(url, { waitUntil: 'load' });
	await page.waitForSelector('main[data-interactive="true"]');
	await new Promise((r) => setTimeout(r, 300));

	const timings = await page.evaluate(() => {
		const nav = performance.getEntriesByType('navigation')[0];
		return {
			responseEnd: nav.responseEnd,
			fcp: performance.getEntriesByName('first-contentful-paint')[0]?.startTime ?? null,
			interactive: performance.getEntriesByName('dashboard-interactive')[0]?.startTime ?? null,
			load: nav.loadEventEnd,
			htmlStatusTime: window.__htmlStatusTime
		};
	});
	runs.push({ run, ...timings, requests });
	console.log(`run ${run}: interactive after ${Math.round(Math.max(timings.fcp, timings.interactive))} ms`);

	await context.close();
	if (run === 1) statsAfterWarmup = await stats();
}
await browser.close();

const statsEnd = await stats();
const measured = RUNS - 1;
const result = {
	mode,
	url,
	date: new Date().toISOString(),
	throttling: { ...NETWORK, cpuSlowdown: CPU_SLOWDOWN },
	serverCpuMsPerLoad: (statsEnd.cpuMs - statsAfterWarmup.cpuMs) / measured,
	serverRssMb: statsEnd.rssMb,
	runs
};

mkdirSync('results', { recursive: true });
writeFileSync(`results/v1-chrome-${mode}.json`, JSON.stringify(result, null, 2));
console.log(`server CPU per page load: ${result.serverCpuMsPerLoad.toFixed(1)} ms`);
