<script lang="ts">
	let { size = 32, class: klass = "" }: { size?: number; class?: string } = $props();
</script>

<svg viewBox="0 0 100 100" width={size} height={size} class={klass} aria-label="Luxmc logo" role="img">
	<defs>
		<linearGradient id="logoFrame" x1="18%" y1="8%" x2="82%" y2="92%">
			<stop offset="0" stop-color="#607582"/>
			<stop offset="1" stop-color="#263540"/>
		</linearGradient>
		<linearGradient id="logoMark" x1="20%" y1="0" x2="80%" y2="100%">
			<stop offset="0" stop-color="#d8ffff"/>
			<stop offset="0.5" stop-color="#63dce0"/>
			<stop offset="1" stop-color="#37aebc"/>
		</linearGradient>
		<filter id="logoGlow" x="-30%" y="-30%" width="160%" height="160%">
			<feGaussianBlur stdDeviation="2" result="blur"/>
			<feMerge><feMergeNode in="blur"/><feMergeNode in="SourceGraphic"/></feMerge>
		</filter>
	</defs>
	<path d="M50 5 88 22v35c0 19-13 31-38 38C25 88 12 76 12 57V22L50 5Z" fill="#1f2b35" stroke="url(#logoFrame)" stroke-width="6" stroke-linejoin="round"/>
	<path d="M47 30 47 70 56 70 56 51 68 70 76 70 76 31 67 31 67 51 55 31Z" fill="url(#logoMark)" filter="url(#logoGlow)"/>
	<path d="M27 35v35h20v-9H36V35Z" fill="url(#logoMark)" filter="url(#logoGlow)"/>
	<path d="m50 4 4 13 13 4-13 4-4 13-4-13-13-4 13-4Z" fill="#d8ffff" filter="url(#logoGlow)"/>
</svg>
