<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const s = $derived(data.status);
	let interactive = $state(false);

	// Runs once the page is rendered/hydrated in the browser, i.e. the buttons work
	$effect(() => {
		interactive = true;
		performance.mark('dashboard-interactive');
	});
</script>

<svelte:head>
	<title>TravelPi Dashboard (Testseite)</title>
</svelte:head>

<main data-interactive={interactive}>
	<h1>TravelPi</h1>
	<section class="cards">
		<article><h2>Internet</h2><p>{s.internetSource}</p></article>
		<article>
			<h2>VPN-Tunnel</h2>
			<p>{s.tunnel.active ? 'aktiv' : 'inaktiv'} · {s.tunnel.profile}</p>
			<button disabled={!interactive}>{s.tunnel.active ? 'Ausschalten' : 'Einschalten'}</button>
		</article>
		<article>
			<h2>Kill-Switch</h2>
			<p>{s.killSwitch ? 'an' : 'aus'}</p>
			<button disabled={!interactive}>Umschalten</button>
		</article>
		<article><h2>Blockiert</h2><p>{s.blockedRequests} Anfragen</p></article>
		<article><h2>Akku</h2><p>{s.battery ?? '–'} %</p></article>
		<article>
			<h2>Hardware</h2>
			<p>{s.hardware.model}</p>
			<p>CPU {s.hardware.cpuLoad} % · RAM {s.hardware.ramUsedMb} MB · {s.hardware.temperature} °C</p>
		</article>
	</section>
	<section>
		<h2>WLAN-Netzwerke</h2>
		<ul>
			{#each s.networks as n (n.ssid)}
				<li><span>{n.ssid}</span><span>{n.signal} dBm {n.secured ? '🔒' : ''}</span></li>
			{/each}
		</ul>
	</section>
	<p class="status-time" data-status-time={s.time}>Stand: {s.time}</p>
</main>

<style>
	main { font-family: system-ui, sans-serif; max-width: 960px; margin: 0 auto; padding: 16px; }
	.cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px; }
	article { border: 1px solid #ccc; border-radius: 8px; padding: 12px; }
	h2 { font-size: 1rem; margin: 0 0 4px; }
	ul { list-style: none; padding: 0; }
	li { display: flex; justify-content: space-between; padding: 6px 0; border-bottom: 1px solid #eee; }
	.status-time { color: #666; font-size: 0.8rem; }
</style>
