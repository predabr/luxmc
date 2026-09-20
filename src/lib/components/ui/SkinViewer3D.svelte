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
    let unavailable = $state(false);
	let resizeObserver: ResizeObserver | null = null;
	let capeCache = new Map<string, HTMLCanvasElement | string>();
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

		if (viewer.animation) {
			viewer.animation.addAnimation((player, progress) => {
				if (!player.cape) return;
				const t = progress * (anim === "run" ? 8 : anim === "walk" ? 5 : 3);
				const baseAngle = anim === "run" ? 0.72 : anim === "walk" ? 0.34 : 0.18;
				const wave1 = Math.sin(t) * (anim === "run" ? 0.22 : anim === "walk" ? 0.14 : 0.08);
				const wave2 = Math.sin(t * 2.2) * (anim === "run" ? 0.08 : anim === "walk" ? 0.04 : 0.025);
				player.cape.rotation.x = baseAngle + wave1 + wave2;
				player.cape.rotation.y = Math.sin(t * 0.65) * (anim === "run" ? 0.08 : 0.04);
				player.cape.rotation.z = Math.cos(t * 0.85) * (anim === "run" ? 0.06 : 0.03);
			});
		}
	}

	onMount(() => {
		if (!canvasEl || !containerEl) return;

		const width = containerEl.clientWidth || 300;
		const height = containerEl.clientHeight || 400;

		try {
        viewer = new SkinViewer({
			canvas: canvasEl,
			width,
			height,
			model: slim ? "slim" : "default",
			enableControls: true
		});
        } catch { unavailable = true; return; }

		if (viewer.controls) {
			viewer.controls.enableRotate = true;
			viewer.controls.enableZoom = true;
			viewer.controls.enablePan = false;
		}

		viewer.camera.position.set(0, 18, 55);
		viewer.camera.lookAt(0, 8, 0);

		updateSkin();
		viewer.autoRotate = autoRotate;
		viewer.autoRotateSpeed = 0.4;
		applyAnimation(animation);
		viewer.playerObject.rotation.y = (15 * Math.PI) / 180;
		viewer.playerObject.skin.setOuterLayerVisible(true);

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
		capeCache.clear();
		capeCache = new Map();
	});

	function updateSkin() {
		if (!viewer) return;
		const gen = ++loadGeneration;
		const targetSkin = skinUrl && skinUrl.trim() ? skinUrl.trim() : "https://minotar.net/skin/Steve";
		
		try {
			const res = viewer.loadSkin(targetSkin, { model: slim ? "slim" : "default" });
			if (res && typeof (res as Promise<void>).then === "function") {
				(res as Promise<void>)
					.then(() => {
						if (gen === loadGeneration && viewer) {
							viewer.playerObject.skin.setOuterLayerVisible(true);
						}
					})
					.catch((e: unknown) => {
						console.warn("Failed to load skin in 3D viewer, falling back to Steve:", e);
						if (gen === loadGeneration && viewer && targetSkin !== "https://minotar.net/skin/Steve") {
							viewer.loadSkin("https://minotar.net/skin/Steve", { model: "default" });
						}
					});
			} else if (viewer) {
				viewer.playerObject.skin.setOuterLayerVisible(true);
			}
		} catch (err) {
			console.warn("Error calling loadSkin:", err);
		}
	}


	function loadAndHealCape(src: string): Promise<HTMLCanvasElement | string> {
		if (!src) return Promise.resolve(src);
		const cached = capeCache.get(src);
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

					// Standard OptiFine cape: 22x17 -> place at (0,0) on 64x32 canvas without stretching
					if (w === 22 && h === 17) {
						const canvas = document.createElement("canvas");
						canvas.width = 64;
						canvas.height = 32;
						const ctx = canvas.getContext("2d");
						if (ctx) {
							ctx.imageSmoothingEnabled = false;
							ctx.drawImage(img, 0, 0);
							capeCache.set(src, canvas);
							resolve(canvas);
							return;
						}
					}

					// HD OptiFine cape: 44x34 -> place at (0,0) on 128x64 canvas
					if (w === 44 && h === 34) {
						const canvas = document.createElement("canvas");
						canvas.width = 128;
						canvas.height = 64;
						const ctx = canvas.getContext("2d");
						if (ctx) {
							ctx.imageSmoothingEnabled = false;
							ctx.drawImage(img, 0, 0);
							capeCache.set(src, canvas);
							resolve(canvas);
							return;
						}
					}

					// Square cape: crop top half
					if (w === h && w > 0) {
						const canvas = document.createElement("canvas");
						canvas.width = w;
						canvas.height = w / 2;
						const ctx = canvas.getContext("2d");
						if (ctx) {
							ctx.imageSmoothingEnabled = false;
							ctx.drawImage(img, 0, 0, w, w / 2, 0, 0, w, w / 2);
							capeCache.set(src, canvas);
							resolve(canvas);
							return;
						}
					}

					capeCache.set(src, src);
					resolve(src);
				} catch {
					resolve(src);
				}
			};
			img.onerror = () => resolve(src);
			img.src = src;
		});
	}

	function applyCrispCapeTexture() {
		try {
			const capeMesh = (viewer?.playerObject as any)?.cape?.mesh;
			if (capeMesh) {
				const mat = Array.isArray(capeMesh.material) ? capeMesh.material[0] : capeMesh.material;
				if (mat && mat.map) {
					mat.map.magFilter = 1003; // NearestFilter
					mat.map.minFilter = 1003; // NearestFilter
					mat.map.generateMipmaps = false;
					mat.map.needsUpdate = true;
				}
			}
		} catch {}
	}

	function updateCape() {
		if (!viewer) return;
		if (cape === "custom" && customCapeUrl) {
			viewer.playerObject.backEquipment = "cape";
			loadAndHealCape(customCapeUrl).then((healed) => {
				if (!viewer) return;
				const res = viewer.loadCape(healed as any, { backEquipment: "cape" });
				if (res && typeof (res as Promise<void>).then === "function") {
					(res as Promise<void>)
						.then(() => {
							applyCrispCapeTexture();
						})
						.catch((e: unknown) => {
							console.warn("Failed to load custom cape in 3D viewer:", e);
						});
				} else {
					applyCrispCapeTexture();
				}
			});
		} else if (cape !== "none") {
			const fullUrl = getFullCapeDataUrl(cape);
			if (fullUrl) {
				viewer.playerObject.backEquipment = "cape";
				loadAndHealCape(fullUrl).then((healed) => {
					if (!viewer) return;
					const res = viewer.loadCape(healed as any, { backEquipment: "cape" });
					if (res && typeof (res as Promise<void>).then === "function") {
						(res as Promise<void>)
							.then(() => {
								applyCrispCapeTexture();
							})
							.catch((e: unknown) => {
								console.warn("Failed to load cape in 3D viewer:", e);
							});
					} else {
						applyCrispCapeTexture();
					}
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
		viewer.camera.position.set(0, 18, 55);
		viewer.camera.lookAt(0, 8, 0);
		viewer.zoom = 1;
		viewer.playerObject.rotation.y = (15 * Math.PI) / 180;
	}
</script>

<div 
	bind:this={containerEl}
	class="relative w-full h-full cursor-grab active:cursor-grabbing select-none overflow-hidden {className}"
	role="region"
	aria-label="Visualizador 3D de Skin"
>
	<canvas bind:this={canvasEl} class="w-full h-full block" class:invisible={unavailable}></canvas>
    {#if unavailable}
        <div class="absolute inset-0 flex flex-col items-center justify-center gap-4 p-6 text-center" role="status">
            <img src={skinUrl || "/grass_head.png"} alt="Textura da skin selecionada" class="h-32 w-32 object-contain [image-rendering:pixelated]" />
            <p class="max-w-xs text-sm text-fg-muted">A prévia 3D precisa de aceleração gráfica. Você pode continuar escolhendo e aplicando skins.</p>
        </div>
    {/if}
	<div class="absolute bottom-2 left-1/2 -translate-x-1/2 w-32 h-6 bg-bg-overlay/40 rounded-full blur-sm pointer-events-none"></div>
</div>
