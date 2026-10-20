import { json } from '@sveltejs/kit';
import { getStatus } from '#lib/status.js';

export const prerender = false;

export function GET() {
	return json(getStatus());
}
