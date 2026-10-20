// Mock status of a TravelPi. Test data only, not read from the device.

const NETWORK_COUNT = 30;

export function getStatus(now = Date.now()) {
	const t = Math.floor(now / 1000);
	return {
		time: new Date(now).toISOString(),
		internetSource: 'Hotel-WLAN',
		tunnel: { active: true, profile: 'wireguard-home' },
		killSwitch: true,
		blockedRequests: t % 100000,
		battery: 80,
		hardware: { model: 'Raspberry Pi', cpuLoad: 20 + (t % 10), ramUsedMb: 412, temperature: 52 },
		networks: Array.from({ length: NETWORK_COUNT }, (_, i) => ({
			ssid: `Hotel-Gast-${String(i + 1).padStart(2, '0')}`,
			signal: -40 - ((i * 7) % 50),
			secured: i % 3 !== 0
		}))
	};
}
