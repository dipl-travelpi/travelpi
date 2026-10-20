import adapter from '@sveltejs/adapter-node';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Build one rendering mode: VITE_RENDER_MODE=csr|ssr|ssg npx vite build -> build-<mode>/
const mode = process.env.VITE_RENDER_MODE ?? 'ssr';

export default defineConfig({
	plugins: [
		sveltekit({
			compilerOptions: {
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			adapter: adapter({ out: `build-${mode}` })
		})
	]
});
