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
			model: slim ? "slim" : "default"
		});

		updateSkin();
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

		const isTrulyEmpty = (a: number) => a < 10;

		const healRegion = (startX: number, startY: number, width: number, height: number, mirrorX: number, mirrorY: number, altX: number, altY: number) => {
			for (let dy = 0; dy < height * scale; dy++) {
				for (let dx = 0; dx < width * scale; dx++) {
					const bx = startX * scale + dx;
					const by = startY * scale + dy;
					const [, , , a] = getPixel(bx, by);
					if (!isTrulyEmpty(a)) continue;
					const [or, og, ob, oa] = getPixel(mirrorX * scale + dx, mirrorY * scale + dy);
					if (oa > 10) {
						setPixel(bx, by, or, og, ob, 255);
					} else if (altX >= 0) {
						const [ar, ag, ab, aa] = getPixel(altX * scale + dx, altY * scale + dy);
						if (aa > 10) {
							setPixel(bx, by, ar, ag, ab, 255);
						} else {
							setPixel(bx, by, fallbackR, fallbackG, fallbackB, 255);
						}
					} else {
						setPixel(bx, by, fallbackR, fallbackG, fallbackB, 255);
					}
				}
			}
		};

		const armWidth = isSlim ? 3 : 4;
		const rBackStartX = isSlim ? 51 : 52;
		const lBackStartX = isSlim ? 43 : 44;

		healRegion(24, 8, 8, 8, 8, 8, 24, 8);

		healRegion(rBackStartX, 20, armWidth, 12, 44, 20, 36, 20);

		healRegion(lBackStartX, 52, armWidth, 12, 36, 52, 44, 52);

		const rHandStartX = isSlim ? 47 : 48;
		healRegion(rHandStartX, 16, armWidth, 4, 44, 32, 44, 32);

		const lHandStartX = isSlim ? 39 : 40;
		healRegion(lHandStartX, 48, armWidth, 4, 36, 48, 36, 48);

		healRegion(32, 20, 8, 12, 20, 20, 20, 20);

		healRegion(12, 20, 4, 12, 0, 20, 4, 20);

		healRegion(28, 52, 4, 12, 16, 52, 20, 52);

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
					const isPlaceholder = (data[idx] === 45 && data[idx + 1] === 45 && data[idx + 2] === 45)
						|| (data[idx] === 40 && data[idx + 1] === 30 && data[idx + 2] === 25);
					if (data[idx + 3] < 10 || isPlaceholder) {
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

	function loadAndHealSkin(src: string, isSlim: boolean): Promise<HTMLCanvasElement | string> {
		return new Promise((resolve) => {
			if (typeof window === "undefined") {
				resolve(src);
				return;
			}
			const img = new window.Image();
			img.crossOrigin = "anonymous";
			img.onload = () => {
				try {
					const w = img.naturalWidth || img.width;
					const h = img.naturalHeight || img.height;
					if (w <= 0 || h <= 0) {
						resolve(src);
						return;
					}
					const canvas = document.createElement("canvas");
					canvas.width = 64;
					canvas.height = 64;
					const ctx = canvas.getContext("2d", { willReadFrequently: true });
					if (!ctx) {
						resolve(src);
						return;
					}
					ctx.imageSmoothingEnabled = false;

					if (w === 64 && h === 32) {
						ctx.drawImage(img, 0, 0);
						// Mirror left leg from right leg
						for (let dy = 0; dy < 4; dy++) {
							for (let dx = 0; dx < 4; dx++) {
								ctx.drawImage(canvas, 4 + dx, 16 + dy, 1, 1, 20 + (3 - dx), 48 + dy, 1, 1);
								ctx.drawImage(canvas, 8 + dx, 16 + dy, 1, 1, 24 + (3 - dx), 48 + dy, 1, 1);
							}
						}
						for (let dy = 0; dy < 12; dy++) {
							for (let dx = 0; dx < 4; dx++) {
								ctx.drawImage(canvas, 4 + dx, 20 + dy, 1, 1, 20 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 12 + dx, 20 + dy, 1, 1, 28 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 0 + dx, 20 + dy, 1, 1, 24 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 8 + dx, 20 + dy, 1, 1, 16 + (3 - dx), 52 + dy, 1, 1);
							}
						}
						// Mirror left arm from right arm
						for (let dy = 0; dy < 4; dy++) {
							for (let dx = 0; dx < 4; dx++) {
								ctx.drawImage(canvas, 44 + dx, 16 + dy, 1, 1, 36 + (3 - dx), 48 + dy, 1, 1);
								ctx.drawImage(canvas, 48 + dx, 16 + dy, 1, 1, 40 + (3 - dx), 48 + dy, 1, 1);
							}
						}
						for (let dy = 0; dy < 12; dy++) {
							for (let dx = 0; dx < 4; dx++) {
								ctx.drawImage(canvas, 44 + dx, 20 + dy, 1, 1, 36 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 52 + dx, 20 + dy, 1, 1, 44 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 40 + dx, 20 + dy, 1, 1, 40 + (3 - dx), 52 + dy, 1, 1);
								ctx.drawImage(canvas, 48 + dx, 20 + dy, 1, 1, 32 + (3 - dx), 52 + dy, 1, 1);
							}
						}
					} else {
						ctx.drawImage(img, 0, 0, 64, 64);
					}

					healSkinCanvas(canvas, isSlim);
					resolve(canvas);
				} catch (err) {
					console.warn("Failed to preprocess skin canvas:", err);
					resolve(src);
				}
			};
			img.onerror = () => {
				resolve(src);
			};
			img.src = src;
		});
	}

	function updateSkin() {
		if (!viewer) return;
		const targetSkin = skinUrl && skinUrl.trim() ? skinUrl : "https://minotar.net/skin/Steve";
		loadAndHealSkin(targetSkin, slim).then((healedSrc) => {
			if (!viewer) return;
			const res = viewer.loadSkin(healedSrc, { model: slim ? "slim" : "default" });
			if (res && typeof (res as Promise<void>).then === "function") {
				(res as Promise<void>)
					.then(() => {
						viewer?.playerObject.skin.setOuterLayerVisible(true);
					})
					.catch((e: unknown) => {
						console.warn("Failed to load skin in 3D viewer:", e);
					});
			} else {
				viewer?.playerObject.skin.setOuterLayerVisible(true);
			}
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
