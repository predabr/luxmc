<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { onMount } from "svelte";
	import { authResolveTexture } from "$lib/api/auth";
	import type { SkinViewer } from "skinview3d";
	import type { CapeType } from "$lib/stores/skin.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { getFullCapeDataUrl } from "$lib/utils/capeTextures";
	import { loadTextureImage, normalizeCape, classifyTexture, textureCanvas, inferSkinModelType } from "$lib/utils/textureImage";

	import { createAnimationClock } from "$lib/utils/animationClock";
	import { createCapeCloth } from "$lib/utils/capeCloth";
	import Skin2DPreview from "./Skin2DPreview.svelte";

	export type AnimationType = "idle" | "walk" | "run" | "fly" | "none";

	let {
		skinUrl = "/steve.png",
		cape = "none",
		customCapeUrl = "",
		slim = false,
		autoRotate = true,
		animation = "walk",
		className = "",
		active = true
	}: {
		skinUrl?: string;
		cape?: CapeType;
		customCapeUrl?: string;
		slim?: boolean;
		autoRotate?: boolean;
		animation?: AnimationType;
		className?: string;
		active?: boolean;
	} = $props();

	let containerEl = $state<HTMLDivElement | null>(null);
	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let viewer = $state.raw<SkinViewer | null>(null);
	let library = $state.raw<typeof import("skinview3d") | null>(null);
	let visible = $state(false);
	let foreground = $state(true);
	let ready = $state(false);
	let hasMountedViewer = $state(false);
	let unavailable = $state(false);
	let textureError = $state(false);
    let capeError = $state(false);
    let capeCloth: ReturnType<typeof createCapeCloth> | null = null;
	let fallbackUsed = $state(false);
	let lastTexture = $state("");

	let dragPointerId: number | null = null;
	let lastPointerX = 0;
	let lastPointerY = 0;

	const running = $derived(active && visible && foreground && !(settings.value.pauseSkinWhileGaming && appState.isGameRunning) && !appState.performanceMode);

	onMount(() => {
		let disposed = false;
		const visibility = () => {
			foreground = !document.hidden;
			if (!foreground) stopDragging();
		};
		visibility();
		document.addEventListener("visibilitychange", visibility);
		window.addEventListener("blur", stopDragging);

		const contextLost = (event: Event) => {
			event.preventDefault();
			unavailable = true;
			ready = false;
		};
		const contextRestored = () => {
			requestAnimationFrame(() => {
				if (!disposed && viewer) {
					unavailable = false;
					ready = renderViewer(viewer);
				}
			});
		};

		canvasEl?.addEventListener("webglcontextlost", contextLost);
		canvasEl?.addEventListener("webglcontextrestored", contextRestored);

		const intersection = new IntersectionObserver((entries) => {
			visible = entries.some((entry) => entry.isIntersecting);
		});
		if (containerEl) intersection.observe(containerEl);

		const resize = new ResizeObserver((entries) => {
			const box = entries[0]?.contentRect;
			if (box && box.width > 0 && box.height > 0 && viewer) {
				viewer.setSize(box.width, box.height);
				if (!unavailable) renderViewer(viewer);
			}
		});
		if (containerEl) resize.observe(containerEl);

		void import("skinview3d").then((module) => {
			if (disposed || !canvasEl || !containerEl) return;
			library = module;
			createViewer();
		}).catch(() => {
			if (!disposed) unavailable = true;
		});

		return () => {
			disposed = true;
			intersection.disconnect();
			resize.disconnect();
			document.removeEventListener("visibilitychange", visibility);
			window.removeEventListener("blur", stopDragging);
			stopDragging();
			canvasEl?.removeEventListener("webglcontextlost", contextLost);
			canvasEl?.removeEventListener("webglcontextrestored", contextRestored);
			disposeViewer();
		};
	});

	function renderViewer(target: SkinViewer): boolean {
		if (target.disposed) return false;
		try {
			if (target.renderer.getContext().isContextLost()) throw new Error(uiText("ui.fcc0b41d0274c482"));
			target.renderer.setRenderTarget(null);
			target.renderer.render(target.scene, target.camera);
			return true;
		} catch (error) {
			if (!unavailable) console.error("Luxmc skin preview:", error);
			unavailable = true;
			return false;
		}
	}

	function applyTextureMaterial(objectRoot: SkinViewer["playerObject"]["skin"] | SkinViewer["playerObject"]["cape"]) {
		const texture = objectRoot.map as unknown as import("three").Texture | null;
		if (!texture) return;
        texture.colorSpace = "srgb";
        texture.needsUpdate = true;
		objectRoot.traverse(object => {
			const mesh = object as unknown as import("three").Mesh;
			if (!mesh.isMesh) return;
			for (const material of Array.isArray(mesh.material) ? mesh.material : [mesh.material]) {
				const skinMaterial = material as import("three").MeshStandardMaterial;
				if (skinMaterial.type !== "MeshStandardMaterial" || skinMaterial.map !== texture) continue;
				skinMaterial.color.setRGB(0, 0, 0);
				skinMaterial.emissive.setRGB(1, 1, 1);
				skinMaterial.emissiveMap = texture;
				skinMaterial.emissiveIntensity = 1;
                skinMaterial.toneMapped = false;
                skinMaterial.depthWrite = !skinMaterial.transparent;
                if (skinMaterial.transparent) skinMaterial.alphaTest = 0.01;
				skinMaterial.needsUpdate = true;
			}
		});
	}

	function disposeViewer() {
		const target = viewer;
		viewer = null;
		if (!target) return;
		const geometries = new Set<import("three").BufferGeometry>();
		const materials = new Set<import("three").Material>();
		target.scene.traverse(object => {
			const mesh = object as unknown as import("three").Mesh;
			if (mesh.geometry) geometries.add(mesh.geometry);
			if (mesh.material) for (const material of Array.isArray(mesh.material) ? mesh.material : [mesh.material]) materials.add(material);
		});
		target.dispose();
		target.composer.dispose();
		geometries.forEach(geometry => geometry.dispose());
		materials.forEach(material => material.dispose());
	}

	function createViewer() {
		if (!library || !canvasEl || !containerEl) return;
		stopDragging();
		disposeViewer();
		ready = false;
		unavailable = false;
		try {
			const target = new library.SkinViewer({
				canvas: canvasEl,
				width: containerEl.clientWidth || 300,
				height: containerEl.clientHeight || 400,
				renderPaused: true,
				preserveDrawingBuffer: false,
				pixelRatio: Math.min(window.devicePixelRatio || 1, 2),
				enableControls: false
			});
			target.controls.dispose();
            canvasEl.style.touchAction = "pan-y";
			target.renderer.setClearColor(0, 0);
			target.cameraLight.intensity = 0;
			target.globalLight.intensity = 0;

			target.fov = 46;
			viewer = target;
			resetCamera();
		} catch (error) {
			console.error("Luxmc skin preview initialization:", error);
			unavailable = true;
		}
	}

	function handlePointerDown(e: PointerEvent) {
		if (!viewer || unavailable || e.button !== 0 || dragPointerId !== null) return;
		try {
			(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		} catch { return; }
		dragPointerId = e.pointerId;
		lastPointerX = e.clientX;
		lastPointerY = e.clientY;
	}

	function handlePointerMove(e: PointerEvent) {
		if (dragPointerId !== e.pointerId || !viewer || unavailable) return;
		if (e.pointerType === "mouse" && e.buttons === 0) {
			stopDragging();
			return;
		}
		const dx = e.clientX - lastPointerX;
		const dy = e.clientY - lastPointerY;
		lastPointerX = e.clientX;
		lastPointerY = e.clientY;
		if (Number.isFinite(dx)) viewer.playerWrapper.rotation.y += dx * 0.012;
		if (Number.isFinite(dy)) viewer.playerObject.rotation.x = Math.max(-0.5, Math.min(0.5, viewer.playerObject.rotation.x + dy * 0.008));
		renderViewer(viewer);
	}

	function stopDragging() {
		const pointerId = dragPointerId;
		dragPointerId = null;
		try {
			if (pointerId !== null && canvasEl?.hasPointerCapture(pointerId)) canvasEl.releasePointerCapture(pointerId);
		} catch {}
	}

	function handlePointerUp(e: PointerEvent) {
		if (dragPointerId === e.pointerId) stopDragging();
	}

	function handleWheel(e: WheelEvent) {
		if (!viewer || unavailable) return;
		if (!e.ctrlKey && !e.metaKey) return;
        e.preventDefault();
		viewer.zoom = Math.max(0.5, Math.min(2.2, viewer.zoom - e.deltaY * 0.0015));
		renderViewer(viewer);
	}

	let isLoadingSkin = $state(false);

	$effect(() => {
		const target = viewer;
		const source = skinUrl || "/steve.png";
		const selectedSlim = slim;
		const model = selectedSlim ? "slim" : "default";
		if (!target) return;
		stopDragging();
		const controller = new AbortController();
		textureError = false;
		fallbackUsed = false;
        isLoadingSkin = true;

		const load = async () => {
			const candidates: Array<() => Promise<string>> = [
				async () => (source.startsWith("https://") ? await authResolveTexture(source) : source),
				async () => source.startsWith("https://") ? source : lastTexture || (selectedSlim ? "/alex.png" : "/steve.png"),
				async () => (selectedSlim ? "/alex.png" : "/steve.png")
			];
			let failure: unknown;
			for (let index = 0; index < candidates.length; index++) {
				try {
					const url = await candidates[index]();
					if (controller.signal.aborted) return;
					const image = await loadTextureImage(url, controller.signal, 5000);
					if (controller.signal.aborted) return;
					if (image.naturalWidth > 2048 || image.naturalHeight > 2048) throw new Error("Textura muito grande");
					if (classifyTexture(textureCanvas(image)) !== "skin") throw new Error(uiText("ui.905da204b0bceda1"));
					try {
						await target.loadSkin(image, { model });
						if (controller.signal.aborted) return;
						applyTextureMaterial(target.playerObject.skin);
						target.playerObject.skin.visible = true;
						target.playerObject.visible = true;
						target.playerObject.skin.setOuterLayerVisible(true);
						ready = renderViewer(target);
						if (!ready) throw new Error(uiText("ui.4b64ead299549268"));
						hasMountedViewer = true;
						fallbackUsed = index > 0;
					} finally {
						if (!controller.signal.aborted) isLoadingSkin = false;
					}
					try {
						const cache = document.createElement("canvas");
						cache.width = image.naturalWidth;
						cache.height = image.naturalHeight;
						cache.getContext("2d")?.drawImage(image, 0, 0);
						lastTexture = cache.toDataURL("image/png");
					} catch {}
					return;
				} catch (error) {
					if (controller.signal.aborted) return;
					failure = error;
				}
			}
			throw failure;
		};

		void load().catch(() => {
			if (!controller.signal.aborted) {
				if (!lastTexture) ready = false;
				textureError = true;
			}
		}).finally(() => { if (!controller.signal.aborted) isLoadingSkin = false; });
		return () => {
			isLoadingSkin = false;
			controller.abort();
		};
	});

	$effect(() => {
		const target = viewer;
		const source = cape === "custom" ? customCapeUrl : cape === "none" ? "" : getFullCapeDataUrl(cape);
		if (!target) return;
		if (!source) {
            capeError = false;
			target.resetCape();
			try { renderViewer(target); } catch {}
			return;
		}
        const controller = new AbortController();
        capeError = false;
        target.resetCape();
        const load = async () => {
            const url = source.startsWith("https://") ? await authResolveTexture(source).catch(() => source) : source;
            const image = await loadTextureImage(url, controller.signal);
            if (controller.signal.aborted || target.disposed) return;
            await target.loadCape(normalizeCape(image), { backEquipment: "cape" });
            if (controller.signal.aborted || target.disposed) return;
            capeCloth = createCapeCloth(target.playerObject.cape.cape);
            applyTextureMaterial(target.playerObject.cape);
            renderViewer(target);
        };
        void load().catch(() => {
            if (controller.signal.aborted || target.disposed) return;
            target.resetCape();
            capeError = true;
            renderViewer(target);
        });
		return () => controller.abort();
	});

	$effect(() => {
		if (!viewer || !library) return;
		const constructors: Record<string, any> = {
			idle: library.IdleAnimation,
			walk: library.WalkingAnimation,
			run: library.RunningAnimation,
			fly: library.FlyingAnimation
		};
		try {
			const AnimClass = animation !== "none" && animation in constructors ? constructors[animation] : null;
			viewer.animation = AnimClass ? new AnimClass() : null;
			if (viewer.animation) viewer.animation.speed = 0.8;
			renderViewer(viewer);
		} catch {}
	});

	$effect(() => {
		const target = viewer;
		if (!target || !running || !ready || isLoadingSkin || unavailable) return;

		const hasAnimation = animation !== "none";
		const canRotate = autoRotate;
		const hasCape = cape !== "none";
		if (!hasAnimation && !canRotate && !hasCape) {
			try { renderViewer(target); } catch {}
			return;
		}

		let frame = 0;
		const clock = createAnimationClock(performance.now());

		let capePitch = 0.18;
		let capeRoll = 0.0;
		let capeYaw = 0.0;
		let lastWrapperRotY = target.playerWrapper.rotation.y;

		const renderLoop = (now: number) => {
			if (isLoadingSkin) {
				clock.reset(now);
				if (target) lastWrapperRotY = target.playerWrapper.rotation.y;
				frame = requestAnimationFrame(renderLoop);
				return;
			}
			const delta = clock.take(now);
			if (delta > 0) {
				try {
					if (hasAnimation) {
						target.animation?.update(target.playerObject, delta);
					}
					if (!Number.isFinite(target.playerWrapper.rotation.y)) target.playerWrapper.rotation.y = 0;
					if (canRotate && dragPointerId === null) {
						target.playerWrapper.rotation.y += delta * 0.45;
					}

					if (hasCape && target.playerObject?.cape && target.playerObject.cape.visible) {
						const currentWrapperRotY = target.playerWrapper.rotation.y;
						let deltaRotY = currentWrapperRotY - lastWrapperRotY;
						lastWrapperRotY = currentWrapperRotY;
						deltaRotY = Number.isFinite(deltaRotY) ? Math.atan2(Math.sin(deltaRotY), Math.cos(deltaRotY)) : 0;
						if (Math.abs(deltaRotY) > 0.6) deltaRotY = Math.sign(deltaRotY) * 0.6;

						const targetPitch = animation === "run"
							? 0.65 + Math.sin(now * 0.014) * 0.08
							: animation === "walk"
							? 0.35 + Math.sin(now * 0.008) * 0.05
							: animation === "fly"
							? 0.85 + Math.sin(now * 0.005) * 0.03
							: 0.18 + Math.sin(now * 0.002) * 0.02;

						const targetRoll = Math.max(-0.4, Math.min(0.4, -deltaRotY * 1.5));
						const targetYaw = Math.max(-0.3, Math.min(0.3, -deltaRotY * 1.2));

						const decayFactor = 1 - Math.exp(-12 * delta);
						capePitch += (targetPitch - capePitch) * decayFactor;
						capeRoll += (targetRoll - capeRoll) * decayFactor;
						capeYaw += (targetYaw - capeYaw) * decayFactor;

						if (!Number.isFinite(capePitch)) capePitch = 0.18;
						if (!Number.isFinite(capeRoll)) capeRoll = 0.0;
						if (!Number.isFinite(capeYaw)) capeYaw = 0.0;

                        capeCloth?.update(now / 1000, animation === "run" ? 1 : animation === "walk" ? .45 : animation === "fly" ? .8 : 0, deltaRotY, Math.max(0,Math.min(1,(settings.value.capeWindStrength ?? 50)/100)), settings.value.capePhysics !== false);
						target.playerObject.cape.rotation.x = Math.max(0.05, Math.min(1.2, capePitch));
						target.playerObject.cape.rotation.z = Math.max(-0.4, Math.min(0.4, capeRoll));
						target.playerObject.cape.rotation.y = Math.PI + Math.max(-0.35, Math.min(0.35, capeYaw));
					}

					renderViewer(target);
				} catch (error) {
					console.error("Luxmc skin animation:", error);
					unavailable = true;
					return;
				}
			}
			frame = requestAnimationFrame(renderLoop);
		};

		frame = requestAnimationFrame(renderLoop);
		return () => cancelAnimationFrame(frame);
	});

	export function setAngle(deg: number) {
		if (viewer) {
			viewer.playerWrapper.rotation.y = (deg * Math.PI) / 180;
			renderViewer(viewer);
		}
	}

	export function setFrontView() {
		if (!viewer) return;
		viewer.playerWrapper.rotation.set(0, 0, 0);
		viewer.playerObject.rotation.set(0, 0, 0);
		renderViewer(viewer);
	}

	export function setBackView() {
		if (!viewer) return;
		viewer.playerWrapper.rotation.set(0, Math.PI, 0);
		viewer.playerObject.rotation.set(0, 0, 0);
		renderViewer(viewer);
	}

	export function setIsometricView() {
		if (!viewer) return;
		viewer.playerWrapper.rotation.set(0, Math.PI / 4, 0);
		viewer.playerObject.rotation.set(0.1, 0, 0);
		renderViewer(viewer);
	}

	export function zoomIn() {
		if (viewer) {
			viewer.zoom = Math.min(2.2, viewer.zoom + 0.2);
			renderViewer(viewer);
		}
	}

	export function zoomOut() {
		if (viewer) {
			viewer.zoom = Math.max(0.5, viewer.zoom - 0.2);
			renderViewer(viewer);
		}
	}

	export function resetView() {
		resetCamera();
	}

	export function dispose() {
		stopDragging();
		disposeViewer();
		ready = false;
	}

	export function resetCamera() {
		if (!viewer) return;
		viewer.camera.position.set(0, 2, 48);
		viewer.camera.lookAt(0, 0, 0);
		viewer.zoom = 0.95;
		viewer.playerObject.position.set(0, 0, 0);
		viewer.playerObject.rotation.set(0, 0, 0);
		viewer.playerWrapper.rotation.set(0, Math.PI / 14, 0);
		renderViewer(viewer);
	}
</script>

<div
	bind:this={containerEl}
	class="relative w-full h-full cursor-grab active:cursor-grabbing select-none overflow-hidden {className}"
	role="region"
	aria-label={uiText("ui.1c1c2ce7dd2b664c")}
>
	<canvas
		bind:this={canvasEl}
		class="w-full h-full block touch-pan-y"
		class:invisible={unavailable || !ready}
		onpointerdown={handlePointerDown}
		onpointermove={handlePointerMove}
		onpointerup={handlePointerUp}
		onpointercancel={handlePointerUp}
		onlostpointercapture={handlePointerUp}
		onwheel={handleWheel}
	></canvas>
    {#if capeError && ready}
        <p role="status" class="absolute bottom-2 left-2 right-2 rounded-xl bg-bg-elevated/95 p-2 text-center text-xs text-fg-muted pointer-events-none">{uiText("ui.5740e1169dfb725f")}</p>
    {/if}
	{#if fallbackUsed && ready}
		<p class="absolute bottom-2 left-2 right-2 text-center text-xs text-fg-muted pointer-events-none">
			{uiText("ui.4fd46e83f5b3b346")}
		</p>
	{/if}
	{#if unavailable || !ready}
		<div class="absolute inset-0 flex flex-col items-center justify-center gap-4 p-6 text-center pointer-events-none" role="status">
			<Skin2DPreview src={skinUrl || (slim ? "/alex.png" : "/steve.png")} model={slim ? "alex" : "steve"} className="h-48" />
			<p class="text-sm text-fg-muted">
				{textureError
					? uiText("ui.d655ffb0cdb020b7")
					: unavailable
						? uiText("ui.b4cbcb90151637bd")
						: uiText("ui.8dcfa22d22226499")}
			</p>
			{#if unavailable}<button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "luxmc-control pointer-events-auto" })} onclick={createViewer}>{uiText("ui.17d7b742f087ae47")}</button>{/if}
		</div>
	{/if}
</div>
