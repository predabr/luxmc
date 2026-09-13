<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { SkinViewer, WalkingAnimation } from "skinview3d";
	import type { CapeType } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";

	type Props = {
		skinUrl?: string;
		cape?: CapeType;
		customCapeUrl?: string;
		slim?: boolean;
		autoRotate?: boolean;
		className?: string;
		active?: boolean;
	};

	let {
		skinUrl = "https://minotar.net/skin/Steve",
		cape = "none",
		customCapeUrl = "",
		slim = false,
		autoRotate = true,
		className = "",
		active = true
	}: Props = $props();

	let containerEl: HTMLDivElement | null = $state(null);
	let canvasEl: HTMLCanvasElement | null = $state(null);
	let viewer: SkinViewer | null = null;
	let resizeObserver: ResizeObserver | null = null;

	onMount(() => {
		if (!canvasEl || !containerEl) return;

		const width = containerEl.clientWidth || 300;
		const height = containerEl.clientHeight || 400;

		viewer = new SkinViewer({
			canvas: canvasEl,
			width,
			height,
			skin: skinUrl || "https://minotar.net/skin/Steve",
			model: slim ? "slim" : "default"
		});

		viewer.autoRotate = autoRotate;
		viewer.autoRotateSpeed = 0.5;
		viewer.animation = new WalkingAnimation();
		viewer.playerObject.rotation.y = (20 * Math.PI) / 180;
		viewer.playerObject.skin.setOuterLayerVisible(true);

		updateCape();

		resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				const { width: w, height: h } = entry.contentRect;
				if (w > 0 && h > 0 && viewer) {
					viewer.setSize(w, h);
				}
			}
		});
		resizeObserver.observe(containerEl);
	});

	onDestroy(() => {
		if (resizeObserver) {
			resizeObserver.disconnect();
			resizeObserver = null;
		}
		if (viewer) {
			viewer.dispose();
			viewer = null;
		}
	});

	function updateSkin() {
		if (!viewer) return;
		const targetSkin = skinUrl && skinUrl.trim() ? skinUrl : "https://minotar.net/skin/Steve";
		viewer.loadSkin(targetSkin, { model: slim ? "slim" : "default" })
			.then(() => {
				viewer?.playerObject.skin.setOuterLayerVisible(true);
			})
			.catch((e) => {
				console.warn("Failed to load skin in 3D viewer:", e);
			});
	}

	function updateCape() {
		if (!viewer) return;
		if (cape === "custom" && customCapeUrl) {
			viewer.playerObject.backEquipment = "cape";
			viewer.loadCape(customCapeUrl, { backEquipment: "cape" }).catch((e) => {
				console.warn("Failed to load custom cape in 3D viewer:", e);
			});
		} else if (cape !== "none") {
			const fullUrl = getFullCapeDataUrl(cape);
			if (fullUrl) {
				viewer.playerObject.backEquipment = "cape";
				viewer.loadCape(fullUrl, { backEquipment: "cape" }).catch((e) => {
					console.warn("Failed to load cape in 3D viewer:", e);
				});
			} else {
				viewer.resetCape();
			}
		} else {
			viewer.resetCape();
		}
	}

	$effect(() => {
		const _s = skinUrl;
		const _m = slim;
		updateSkin();
	});

	$effect(() => {
		const _c = cape;
		const _u = customCapeUrl;
		updateCape();
	});

	$effect(() => {
		if (viewer) {
			viewer.autoRotate = autoRotate;
		}
	});

	$effect(() => {
		if (viewer) {
			viewer.renderPaused = !active;
		}
	});

	export function setAngle(deg: number) {
		if (viewer) {
			viewer.autoRotate = false;
			viewer.playerObject.rotation.y = (deg * Math.PI) / 180;
		}
	}

	export function zoomIn() {
		if (viewer) {
			viewer.zoom = Math.min(2.5, (viewer.zoom || 1) + 0.2);
		}
	}

	export function zoomOut() {
		if (viewer) {
			viewer.zoom = Math.max(0.4, (viewer.zoom || 1) - 0.2);
		}
	}

	export function resetView() {
		if (viewer) {
			viewer.resetCameraPose();
			viewer.zoom = 1;
			viewer.playerObject.rotation.y = (20 * Math.PI) / 180;
		}
	}
</script>

<div 
	bind:this={containerEl}
	class="relative w-full h-full cursor-grab active:cursor-grabbing select-none overflow-hidden {className}"
	role="region"
	aria-label="Visualizador 3D de Skin"
>
	<canvas bind:this={canvasEl} class="w-full h-full block"></canvas>
	<div class="absolute bottom-2 left-1/2 -translate-x-1/2 w-32 h-6 bg-black/40 rounded-full blur-sm pointer-events-none"></div>
</div>
