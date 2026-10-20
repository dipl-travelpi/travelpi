// Rendering mode is chosen at build time: VITE_RENDER_MODE=csr|ssr|ssg
const mode = import.meta.env.VITE_RENDER_MODE ?? 'ssr';

export const ssr = mode !== 'csr';
export const prerender = mode === 'ssg';
