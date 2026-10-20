// Appends each in-browser measurement as one JSON line to results/v1-device-<mode>.jsonl
import { appendFileSync, mkdirSync } from 'node:fs';
import { json } from '@sveltejs/kit';

export const prerender = false;

const mode = import.meta.env.VITE_RENDER_MODE ?? 'ssr';

export async function POST({ request }) {
	const measurement = await request.json();
	mkdirSync('results', { recursive: true });
	appendFileSync(
		`results/v1-device-${mode}.jsonl`,
		JSON.stringify({ mode, received: new Date().toISOString(), ...measurement }) + '\n'
	);
	return json({ ok: true });
}
