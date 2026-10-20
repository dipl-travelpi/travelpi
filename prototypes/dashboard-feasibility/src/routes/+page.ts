import { building } from '$app/env';
import { getStatus } from '#lib/status.js';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	// SSG: no server is running during the build, so the build-time status is embedded
	if (building) return { status: getStatus() };
	const res = await fetch('/api/status');
	return { status: await res.json() };
};
