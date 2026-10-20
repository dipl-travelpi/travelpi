<script lang="ts">
	import favicon from '#lib/assets/favicon.svg';
	import { selfMeasure } from '#lib/self-measure.ts';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();

	$effect(() => {
		selfMeasure();
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
