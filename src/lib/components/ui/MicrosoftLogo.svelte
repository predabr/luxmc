<script lang="ts">
	let { size = 16, class: klass = "" }: { size?: number; class?: string } = $props();
</script>

<svg
	viewBox="0 0 21 21"
	width={size}
	height={size}
	class="shrink-0 {klass}"
	xmlns="http://www.w3.org/2000/svg"
	aria-label="Microsoft"
>
	<rect x="1" y="1" width="9" height="9" fill="#f25022" rx="0.5" />
	<rect x="11" y="1" width="9" height="9" fill="#7fba00" rx="0.5" />
	<rect x="1" y="11" width="9" height="9" fill="#00a4ef" rx="0.5" />
	<rect x="11" y="11" width="9" height="9" fill="#ffb900" rx="0.5" />
</svg>
