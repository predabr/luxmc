<script lang="ts">
	import { T, useTask } from '@threlte/core';
	import { Float } from '@threlte/extras';
	import { onMount } from 'svelte';
	import * as THREE from 'three';

	// Constants for the voxels
	const COUNT = 40;
	
	// Create random data for each voxel
	const voxels = Array.from({ length: COUNT }).map((_, i) => {
		const scale = Math.random() * 0.8 + 0.2;
		return {
			id: i,
			position: [
				(Math.random() - 0.5) * 30, // x spread
				(Math.random() - 0.5) * 30, // y spread
				(Math.random() - 0.5) * 20 - 10, // z depth
			] as [number, number, number],
			scale: [scale, scale, scale] as [number, number, number],
			rotation: [
				Math.random() * Math.PI,
				Math.random() * Math.PI,
				Math.random() * Math.PI,
			] as [number, number, number],
			speed: Math.random() * 0.5 + 0.1,
			color: Math.random() > 0.5 ? 0x2dd4bf : 0x14b8a6,
			opacity: Math.random() * 0.4 + 0.1
		};
	});

	let mouseX = 0;
	let mouseY = 0;
	let targetX = 0;
	let targetY = 0;

	const windowHalfX = typeof window !== 'undefined' ? window.innerWidth / 2 : 0;
	const windowHalfY = typeof window !== 'undefined' ? window.innerHeight / 2 : 0;

	onMount(() => {
		const onDocumentMouseMove = (event: MouseEvent) => {
			mouseX = (event.clientX - windowHalfX) * 0.002;
			mouseY = (event.clientY - windowHalfY) * 0.002;
		};
		
		document.addEventListener('mousemove', onDocumentMouseMove);
		return () => {
			document.removeEventListener('mousemove', onDocumentMouseMove);
		};
	});

	let groupRef: THREE.Group | undefined = $state();

	useTask((delta) => {
		if (!groupRef) return;
		
		// Ease to target mouse
		targetX = targetX + (mouseX - targetX) * 0.02;
		targetY = targetY + (mouseY - targetY) * 0.02;
		
		// Parallax rotation
		groupRef.rotation.x = targetY * 0.5;
		groupRef.rotation.y = targetX * 0.5;
		
		// Slow constant rotation
		groupRef.rotation.y += delta * 0.05;
	});

</script>

<T.Group bind:ref={groupRef}>
	{#each voxels as voxel (voxel.id)}
		<Float speed={voxel.speed} floatIntensity={2} floatingRange={[-1, 1]}>
			<T.Mesh
				position={voxel.position}
				rotation={voxel.rotation}
				scale={voxel.scale}
			>
				<T.BoxGeometry args={[1, 1, 1]} />
				<!-- Glowing glass-like material -->
				<T.MeshPhysicalMaterial
					color={voxel.color}
					transparent={true}
					opacity={voxel.opacity}
					roughness={0.2}
					metalness={0.1}
					transmission={0.9}
					thickness={0.5}
					emissive={voxel.color}
					emissiveIntensity={0.2}
				/>
			</T.Mesh>
		</Float>
	{/each}
</T.Group>

<!-- Lighting -->
<T.AmbientLight intensity={0.5} />
<T.DirectionalLight position={[10, 10, 10]} intensity={1} color="#2dd4bf" />
<T.PointLight position={[-10, -10, -10]} intensity={2} color="#14b8a6" distance={50} />

<!-- Subtle fog to blend with the dark background -->
<T.Fog attach="fog" args={['#0d0e12', 10, 40]} />
