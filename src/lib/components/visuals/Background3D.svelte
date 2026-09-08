<script lang="ts">
	import { Canvas } from '@threlte/core';
	import Scene from './Scene.svelte';
</script>

<div class="fixed inset-0 z-0 pointer-events-none opacity-60">
	<Canvas>
		<Scene />
	</Canvas>
</div>
