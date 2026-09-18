<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { SkinViewer, IdleAnimation, WalkingAnimation, RunningAnimation, FlyingAnimation } from "skinview3d";
	import type { CapeType } from "$lib/stores/skin.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";

	type AnimationType = "idle" | "walk" | "run" | "fly" | "none";

	type Props = {
		skinUrl?: string;
		cape?: CapeType;
		customCapeUrl?: string;
		slim?: boolean;
		autoRotate?: boolean;
		animation?: AnimationType;
		className?: string;
		active?: boolean;
	};

	let {
		skinUrl = "https://minotar.net/skin/Steve",
		cape = "none",
		customCapeUrl = "",
		slim = false,
		autoRotate = true,
		animation = "walk",
		className = "",
		active = true
	}: Props = $props();

	let containerEl: HTMLDivElement | null = $state(null);
	let canvasEl: HTMLCanvasElement | null = $state(null);
	let viewer: SkinViewer | null = null;
	let resizeObserver: ResizeObserver | null = null;
	let skinCache = new Map<string, HTMLCanvasElement | string>();
	let loadGeneration = 0;

	function applyAnimation(anim: AnimationType) {
		if (!viewer) return;
		if (anim === "idle") {
			viewer.animation = new IdleAnimation();
			viewer.animation.speed = 0.8;
		} else if (anim === "walk") {
			viewer.animation = new WalkingAnimation();
			viewer.animation.speed = 0.8;
		} else if (anim === "run") {
			viewer.animation = new RunningAnimation();
			viewer.animation.speed = 0.9;
		} else if (anim === "fly") {
			viewer.animation = new FlyingAnimation();
			viewer.animation.speed = 0.8;
		} else {
			viewer.animation = null;
		}
	}

	onMount(() => {
		if (!canvasEl || !containerEl) return;

		const width = containerEl.clientWidth || 300;
		const height = containerEl.clientHeight || 400;

		viewer = new SkinViewer({
			canvas: canvasEl,
			width,
			height,
			model: slim ? "slim" : "default",
			enableControls: true
		});

		if (viewer.controls) {
			viewer.controls.enableRotate = true;
			viewer.controls.enableZoom = true;
			viewer.controls.enablePan = false;
		}

		viewer.camera.position.set(20, 15, 45);
		viewer.camera.lookAt(0, 0, 0);

		updateSkin();
		viewer.autoRotate = autoRotate;
		viewer.autoRotateSpeed = 0.5;
		applyAnimation(animation);
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
		skinCache.clear();
		skinCache = new Map();
	});

	function loadAndHealSkin(src: string, isSlim: boolean): Promise<HTMLCanvasElement | string> {
		const cacheKey = `${src}|${isSlim ? "slim" : "default"}`;
		const cached = skinCache.get(cacheKey);
		if (cached) return Promise.resolve(cached);

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
					const ctx = canvas.getContext("2d");
					if (!ctx) {
						resolve(src);
						return;
					}
					ctx.imageSmoothingEnabled = false;

					if (h === 32) {
						// Standard 64x32 to 64x64 expansion for legacy Minecraft skins
						ctx.drawImage(img, 0, 0);

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

					if (skinCache.size > 10) {
						const firstKey = skinCache.keys().next().value;
						if (firstKey) skinCache.delete(firstKey);
					}
					skinCache.set(cacheKey, canvas);
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
		const gen = ++loadGeneration;
		const targetSkin = skinUrl && skinUrl.trim() ? skinUrl : "https://minotar.net/skin/Steve";
		loadAndHealSkin(targetSkin, slim).then((healedSrc) => {
			if (!viewer || gen !== loadGeneration) return;
			const res = viewer.loadSkin(healedSrc, { model: slim ? "slim" : "default" });
			if (res && typeof (res as Promise<void>).then === "function") {
				(res as Promise<void>)
					.then(() => {
						if (gen === loadGeneration) viewer?.playerObject.skin.setOuterLayerVisible(true);
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
		const _c = cape;
		const _u = customCapeUrl;
		const _r = autoRotate;
		const _a = active;
		const _anim = animation;

		if (viewer) {
			viewer.autoRotate = _r;
			viewer.renderPaused = !_a;
			applyAnimation(_anim);
		}
		updateSkin();
		updateCape();
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
		resetCamera();
	}

	export function resetCamera() {
		if (!viewer) return;
		viewer.camera.position.set(20, 15, 45);
		viewer.camera.lookAt(0, 0, 0);
		viewer.zoom = 1;
		viewer.playerObject.rotation.y = (20 * Math.PI) / 180;
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
