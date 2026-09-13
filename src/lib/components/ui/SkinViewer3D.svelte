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

		// Balanced lighting: ambient + camera light + backlight so the back and arms are brightly rendered
		viewer.globalLight.intensity = 2.4;
		viewer.cameraLight.intensity = 0.9;
		try {
			const backLight1 = viewer.cameraLight.clone();
			backLight1.position.set(25, 10, -45);
			backLight1.intensity = 0.9;
			viewer.scene.add(backLight1);

			const backLight2 = viewer.cameraLight.clone();
			backLight2.position.set(-25, 10, -45);
			backLight2.intensity = 0.9;
			viewer.scene.add(backLight2);
		} catch {}

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

	function healSkinCanvas(canvas: HTMLCanvasElement, isSlim: boolean) {
		const ctx = canvas.getContext("2d", { willReadFrequently: true });
		if (!ctx) return;
		const w = canvas.width;
		const h = canvas.height;
		if (w <= 0 || h <= 0) return;
		const scale = Math.max(1, Math.floor(w / 64));
		const imgData = ctx.getImageData(0, 0, w, h);
		const data = imgData.data;

		const getPixel = (x: number, y: number): [number, number, number, number] => {
			if (x < 0 || x >= w || y < 0 || y >= h) return [0, 0, 0, 0];
			const idx = (y * w + x) * 4;
			return [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];
		};

		const setPixel = (x: number, y: number, r: number, g: number, b: number, a = 255) => {
			if (x < 0 || x >= w || y < 0 || y >= h) return;
			const idx = (y * w + x) * 4;
			data[idx] = r;
			data[idx + 1] = g;
			data[idx + 2] = b;
			data[idx + 3] = a;
		};

		let fallbackR = 210, fallbackG = 165, fallbackB = 130;
		const samplePoints = [
			[24 * scale, 24 * scale],
			[22 * scale, 22 * scale],
			[12 * scale, 12 * scale],
			[26 * scale, 26 * scale]
		];
		for (const [sx, sy] of samplePoints) {
			const [r, g, b, a] = getPixel(sx, sy);
			if (a > 200 && (r > 60 || g > 60 || b > 60)) {
				fallbackR = r;
				fallbackG = g;
				fallbackB = b;
				break;
			}
		}

		const isProblematic = (r: number, g: number, b: number, a: number) => {
			return a < 200 
				|| (r < 10 && g < 10 && b < 10)
				|| (r === 45 && g === 45 && b === 45)
				|| (r === 40 && g === 30 && b === 25);
		};

		const armWidth = isSlim ? 3 : 4;
		const rBackStartX = isSlim ? 51 : 52;
		const rFrontStartX = 44;
		for (let dy = 0; dy < 12 * scale; dy++) {
			for (let dx = 0; dx < armWidth * scale; dx++) {
				const bx = rBackStartX * scale + dx;
				const by = 20 * scale + dy;
				const [r, g, b, a] = getPixel(bx, by);
				if (isProblematic(r, g, b, a)) {
					const [or, og, ob, oa] = getPixel(bx, by + 16 * scale);
					if (oa > 50 && !(or < 10 && og < 10 && ob < 10)) {
						setPixel(bx, by, or, og, ob, 255);
					} else {
						const [fr, fg, fb, fa] = getPixel(rFrontStartX * scale + dx, by);
						if (fa > 50 && !(fr < 10 && fg < 10 && fb < 10)) {
							setPixel(bx, by, fr, fg, fb, 255);
						} else {
							setPixel(bx, by, fallbackR, fallbackG, fallbackB, 255);
						}
					}
				}
			}
		}

		const lBackStartX = isSlim ? 43 : 44;
		const lFrontStartX = 36;
		const rBackStartXLeft = isSlim ? 51 : 52;
		for (let dy = 0; dy < 12 * scale; dy++) {
			for (let dx = 0; dx < armWidth * scale; dx++) {
				const bx = lBackStartX * scale + dx;
				const by = 52 * scale + dy;
				const [r, g, b, a] = getPixel(bx, by);
				if (isProblematic(r, g, b, a)) {
					const rbx = rBackStartXLeft * scale + (armWidth * scale - 1 - dx);
					const rby = 20 * scale + dy;
					const [rr, rg, rb, ra] = getPixel(rbx, rby);
					if (ra > 50 && !(rr < 10 && rg < 10 && rb < 10)) {
						setPixel(bx, by, rr, rg, rb, 255);
					} else {
						const [fr, fg, fb, fa] = getPixel(lFrontStartX * scale + dx, by);
						if (fa > 50 && !(fr < 10 && fg < 10 && fb < 10)) {
							setPixel(bx, by, fr, fg, fb, 255);
						} else {
							setPixel(bx, by, fallbackR, fallbackG, fallbackB, 255);
						}
					}
				}
			}
		}

		for (let dy = 0; dy < 12 * scale; dy++) {
			for (let dx = 0; dx < 8 * scale; dx++) {
				const bx = 32 * scale + dx;
				const by = 20 * scale + dy;
				const [r, g, b, a] = getPixel(bx, by);
				if (isProblematic(r, g, b, a)) {
					const [or, og, ob, oa] = getPixel(bx, by + 16 * scale);
					if (oa > 50 && !(or < 10 && og < 10 && ob < 10)) {
						setPixel(bx, by, or, og, ob, 255);
					} else {
						const [fr, fg, fb, fa] = getPixel(20 * scale + dx, by);
						if (fa > 50 && !(fr < 10 && fg < 10 && fb < 10)) {
							setPixel(bx, by, fr, fg, fb, 255);
						} else {
							setPixel(bx, by, fallbackR, fallbackG, fallbackB, 255);
						}
					}
				}
			}
		}

		const baseRects = [
			[0 * scale, 0 * scale, 32 * scale, 16 * scale],
			[16 * scale, 16 * scale, 40 * scale, 32 * scale],
			[40 * scale, 16 * scale, 56 * scale, 32 * scale],
			[0 * scale, 16 * scale, 16 * scale, 32 * scale],
			[16 * scale, 48 * scale, 32 * scale, 64 * scale],
			[32 * scale, 48 * scale, 48 * scale, 64 * scale]
		];
		for (const [x1, y1, x2, y2] of baseRects) {
			for (let y = y1; y < Math.min(y2, h); y++) {
				for (let x = x1; x < Math.min(x2, w); x++) {
					const idx = (y * w + x) * 4;
					if (data[idx + 3] > 0 && data[idx + 3] < 250) {
						data[idx + 3] = 255;
					}
					if (data[idx + 3] > 200 && data[idx] < 10 && data[idx + 1] < 10 && data[idx + 2] < 10) {
						data[idx] = fallbackR;
						data[idx + 1] = fallbackG;
						data[idx + 2] = fallbackB;
						data[idx + 3] = 255;
					}
				}
			}
		}

		ctx.putImageData(imgData, 0, 0);
	}

	function updateSkin() {
		if (!viewer) return;
		const targetSkin = skinUrl && skinUrl.trim() ? skinUrl : "https://minotar.net/skin/Steve";
		viewer.loadSkin(targetSkin, { model: slim ? "slim" : "default" })
			.then(() => {
				if (viewer?.skinCanvas) {
					healSkinCanvas(viewer.skinCanvas, slim);
					(viewer as unknown as { recreateSkinTexture(): void }).recreateSkinTexture();
				}
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
