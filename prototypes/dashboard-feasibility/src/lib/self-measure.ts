// In-browser measurement for real devices (e.g. iPhone), enabled by ?run=1&of=16.
// Sends the timings to /api/metrics and reloads the page for the next run.
// All times are in ms since the start of the navigation.

export async function selfMeasure() {
	const params = new URLSearchParams(location.search);
	const run = Number(params.get('run'));
	const of = Number(params.get('of'));
	if (!run || !of) return;

	const interactive = await waitForMark('dashboard-interactive', 10000);
	if (document.readyState !== 'complete') await new Promise((r) => addEventListener('load', r, { once: true }));
	await new Promise((r) => setTimeout(r, 300)); // let the browser record paint entries

	const nav = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined;
	const fcp = performance.getEntriesByName('first-contentful-paint')[0];
	await fetch('/api/metrics', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({
			device: params.get('device') ?? 'unknown',
			run,
			responseEnd: nav?.responseEnd ?? null,
			fcp: fcp?.startTime ?? null, // null if the browser does not support Paint Timing
			interactive,
			load: nav?.loadEventEnd ?? null,
			htmlStatusTime: (window as { __htmlStatusTime?: string }).__htmlStatusTime ?? null,
			userAgent: navigator.userAgent
		})
	});

	showNote(run < of ? `Run ${run}/${of} saved` : `Done: ${of} runs saved`);
	if (run < of) {
		params.set('run', String(run + 1));
		setTimeout(() => location.replace(`${location.pathname}?${params}`), 1000);
	}
}

function waitForMark(name: string, timeout: number): Promise<number | null> {
	const start = Date.now();
	return new Promise((resolve) => {
		(function check() {
			const mark = performance.getEntriesByName(name)[0];
			if (mark) resolve(mark.startTime);
			else if (Date.now() - start > timeout) resolve(null);
			else setTimeout(check, 20);
		})();
	});
}

function showNote(text: string) {
	const el = document.createElement('div');
	el.textContent = text;
	el.style.cssText = 'position:fixed;bottom:8px;left:8px;padding:6px 10px;background:#000;color:#fff;font:12px system-ui';
	document.body.append(el);
}
