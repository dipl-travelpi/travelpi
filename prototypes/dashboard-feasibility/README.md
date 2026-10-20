# Dashboard feasibility tests

Tests for the technical feasibility study of the TravelPi web dashboard (diploma thesis,
TGM Wien). They were run on a Raspberry Pi 3 Model B to compare technologies for the
dashboard:

| Test | Compares | Measures | Criteria |
|---|---|---|---|
| **V1** | Rendering: CSR, SSR, SSG | load time on a phone, server CPU per page load, server load with 5, 10 and 30 devices, whether the first view shows the current status | A, B, E |
| **V2** | Communication: REST polling (1 s, 5 s), Server-Sent Events, WebSocket | delay until a status change reaches the dashboard, server CPU and memory with 1, 5 and 10 clients | B, E |

Criteria (from the thesis): A load time under 3 s, B status shown is current, E resources on the Pi.

## What is in this folder

| Path | What it is |
|---|---|
| `src/` | The test dashboard (SvelteKit): one page with mock status data. This is what is measured. |
| `src/lib/status.js` | Mock status data (tunnel, kill switch, 30 Wi-Fi networks) |
| `src/lib/self-measure.ts` | Lets a real phone measure its own page loads (V1) |
| `server.js` | Serves one build of the dashboard: no caching, compressed responses (build files pre-compressed), `GET /stats` for CPU and memory |
| `tests/v1-chrome.js` | V1: 16 page loads in Chrome with mobile throttling |
| `tests/v1-load.js` | V1: 5, 10 and 30 simulated devices loading the page for 30 s each |
| `tests/v2-server.js` | V2: server on the Pi that toggles the kill switch at random times |
| `tests/v2-clients.js` | V2: clients that report back every change they see |
| `results/` | Measurements from the Raspberry Pi |

Everything else (`package.json`, `vite.config.ts`, `tsconfig.json`, …) is the standard SvelteKit setup.

## Setup

Requirements: Node.js 22.17 or newer on the computer and on the Pi, Google Chrome on the computer
(macOS path in `tests/v1-chrome.js`), computer, phone and Pi in the same network.

On the computer:

```bash
npm install
for mode in csr ssr ssg; do VITE_RENDER_MODE=$mode npx vite build; done
rsync -a --exclude node_modules --exclude .svelte-kit ./ pi@pi.local:dashboard-feasibility/
```

On the Pi:

```bash
cd dashboard-feasibility
npm install --omit=dev
```

## Running the tests

Before and after the tests, check that the Pi is not throttled (`throttled=0x0`):

```bash
vcgencmd get_throttled
```

### V1 (repeat for `csr`, `ssr` and `ssg`)

1. On the Pi, start one mode:
   ```bash
   node server.js ssr
   ```
2. On the computer:
   ```bash
   node tests/v1-chrome.js http://pi.local:3000 ssr
   node tests/v1-load.js http://pi.local:3000 ssr
   ```
3. On the phone, open `http://pi.local:3000/?run=1&of=16&device=iphone` and wait until it shows
   “Done: 16 runs saved”. The results are written on the Pi.
4. Stop the server (Ctrl+C) and continue with the next mode.

### V2 (repeat for `polling-1s`, `polling-5s`, `sse` and `websocket`, about 6 minutes each)

1. On the Pi, start the server (restart it for every method so memory values are comparable):
   ```bash
   node tests/v2-server.js
   ```
2. On the computer:
   ```bash
   node tests/v2-clients.js http://pi.local:3001 sse
   ```
3. Stop the server (Ctrl+C) and continue with the next method.

Finally, copy the results from the Pi into `results/` on the computer:

```bash
scp pi@pi.local:dashboard-feasibility/results/v1-device-*.jsonl results/
```

## Results

| File | Content |
|---|---|
| `v1-chrome-<mode>.json` | per run: `responseEnd`, `fcp`, `interactive`, `load` (ms since navigation start), `htmlStatusTime`, requested files; `serverCpuMsPerLoad` |
| `v1-device-<mode>.jsonl` | the same timings measured on the phone, one line per run |
| `v1-load-<mode>.json` | per number of devices: page loads per second, request latency, server CPU (% of one core, can exceed 100 % on multiple cores), memory |
| `v2-<method>.json` | per number of clients: delay from change to confirmation (median, 95th percentile, max), server CPU, memory |

How the values are evaluated:

- Run 1 of every V1 series is a warm-up and is not evaluated; the median of runs 2–16 is used.
- “Interactive” is the later of `fcp` and `interactive`: the page is visible *and* reacts.
- `htmlStatusTime` is the status time in the delivered HTML. With SSG it is the build time.

## Limitations

- The status data is mocked; reading real system values is not part of the test.
- Chrome throttling (4× CPU, “Slow 4G”, Lighthouse defaults) only approximates a slower phone.
- HTTP without TLS; no reverse proxy, no HTTP/2.
- V2 clients are small Node.js programs, not browsers. Every method confirms changes with the same
  `POST /ack`, so the measured delay includes this way back (a few milliseconds in a local network)
  and the confirmations cost the same CPU for every method.
- Simulated devices in `v1-load.js` load the page continuously; this shows the capacity of the
  Pi, not typical use.
