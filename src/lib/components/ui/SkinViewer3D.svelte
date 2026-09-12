<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import * as THREE from "three";
	import type { CapeType } from "$lib/stores/skin.svelte";
	import { drawCapeToCanvas } from "$lib/utils/capeTextures";

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
	let animFrameId: number | null = null;
	let scene: THREE.Scene;
	let camera: THREE.PerspectiveCamera;
	let renderer: THREE.WebGLRenderer;
	let playerGroup: THREE.Group;
	let capeMesh: THREE.Mesh | null = null;
	let isVisible = false;
	let resizeObserver: ResizeObserver | null = null;

	// Texture and materials
	let currentTexture: THREE.CanvasTexture | THREE.Texture | null = null;
	let solidMaterial: THREE.MeshLambertMaterial;
	let overlayMaterial: THREE.MeshLambertMaterial;

	// Orbit drag state
	let isDragging = false;
	let prevMouseX = 0;
	let prevMouseY = 0;
	let yaw = 20 * (Math.PI / 180);
	let pitch = 4 * (Math.PI / 180);
	let zoom = 36;

	// Minecraft skin UV helper
	function setBoxUV(geometry: THREE.BoxGeometry, x: number, y: number, w: number, h: number, d: number, tw = 64, th = 64) {
		const uv = geometry.getAttribute("uv") as THREE.BufferAttribute;
		function setFace(faceIndex: number, u1: number, v1: number, u2: number, v2: number) {
			const i = faceIndex * 4;
			uv.setXY(i + 0, u1 / tw, 1 - v1 / th);
			uv.setXY(i + 1, u2 / tw, 1 - v1 / th);
			uv.setXY(i + 2, u1 / tw, 1 - v2 / th);
			uv.setXY(i + 3, u2 / tw, 1 - v2 / th);
		}
		// 0: Right (+X), 1: Left (-X), 2: Top (+Y), 3: Bottom (-Y), 4: Front (+Z), 5: Back (-Z)
		setFace(0, x, y + d, x + d, y + d + h);
		setFace(1, x + d + w, y + d, x + d + w + d, y + d + h);
		setFace(2, x + d, y, x + d + w, y + d);
		setFace(3, x + d + w, y, x + d + w + w, y + d);
		setFace(4, x + d, y + d, x + d + w, y + d + h);
		setFace(5, x + d + w + d, y + d, x + d + w + d + w, y + d + h);
		uv.needsUpdate = true;
	}

	function setCapeUV(geometry: THREE.BoxGeometry) {
		const uv = geometry.getAttribute("uv") as THREE.BufferAttribute;
		const tw = 64;
		const th = 32;

		function setFace(faceIndex: number, u1: number, v1: number, u2: number, v2: number) {
			const i = faceIndex * 4;
			uv.setXY(i + 0, u1 / tw, 1 - v1 / th);
			uv.setXY(i + 1, u2 / tw, 1 - v1 / th);
			uv.setXY(i + 2, u1 / tw, 1 - v2 / th);
			uv.setXY(i + 3, u2 / tw, 1 - v2 / th);
		}

		// Standard Minecraft cape box: width=10, height=16, depth=1
		// 0: Right (+X): (0, 1) to (1, 17)
		setFace(0, 0, 1, 1, 17);
		// 1: Left (-X): (11, 1) to (12, 17)
		setFace(1, 11, 1, 12, 17);
		// 2: Top (+Y): (1, 0) to (11, 1)
		setFace(2, 1, 0, 11, 1);
		// 3: Bottom (-Y): (11, 0) to (21, 1)
		setFace(3, 11, 0, 21, 1);
		// 4: Front (+Z, inner face facing player body): (12, 1) to (22, 17)
		setFace(4, 12, 1, 22, 17);
		// 5: Back (-Z, outer face facing viewer with cape design): (1, 1) to (11, 17)
		setFace(5, 1, 1, 11, 17);

		uv.needsUpdate = true;
	}

	const capeTextureCache = new Map<CapeType, THREE.CanvasTexture>();

	function createDefaultSteveCanvas(): HTMLCanvasElement {
		const canvas = document.createElement("canvas");
		canvas.width = 64;
		canvas.height = 64;
		const ctx = canvas.getContext("2d");
		if (!ctx) return canvas;
		ctx.imageSmoothingEnabled = false;
		ctx.fillStyle = "#2c1c11";
		ctx.fillRect(0, 0, 64, 64);
		ctx.fillStyle = "#c68c62";
		ctx.fillRect(8, 8, 8, 8);
		ctx.fillStyle = "#2e1b12";
		ctx.fillRect(8, 8, 8, 2);
		ctx.fillStyle = "#ffffff";
		ctx.fillRect(8, 12, 2, 1);
		ctx.fillRect(14, 12, 2, 1);
		ctx.fillStyle = "#334c9c";
		ctx.fillRect(9, 12, 1, 1);
		ctx.fillRect(14, 12, 1, 1);
		ctx.fillStyle = "#8c5638";
		ctx.fillRect(10, 13, 4, 1);
		ctx.fillStyle = "#4a2d1d";
		ctx.fillRect(11, 14, 2, 1);
		ctx.fillStyle = "#00839e";
		ctx.fillRect(16, 16, 24, 16);
		ctx.fillRect(40, 16, 16, 16);
		ctx.fillRect(32, 48, 16, 16);
		ctx.fillStyle = "#27347a";
		ctx.fillRect(0, 16, 16, 16);
		ctx.fillRect(16, 48, 16, 16);
		return canvas;
	}

	function createCapeTexture(type: CapeType): THREE.CanvasTexture {
		if (capeTextureCache.has(type)) {
			return capeTextureCache.get(type)!;
		}

		const canvas = document.createElement("canvas");
		canvas.width = 64;
		canvas.height = 32;
		const ctx = canvas.getContext("2d")!;
		drawCapeToCanvas(ctx, type);

		const tex = new THREE.CanvasTexture(canvas);
		tex.magFilter = THREE.NearestFilter;
		tex.minFilter = THREE.NearestFilter;
		tex.generateMipmaps = false;
		capeTextureCache.set(type, tex);
		return tex;
	}

	function disposePlayerGroup(group: THREE.Group) {
		group.traverse((child) => {
			if ((child as THREE.Mesh).isMesh) {
				const mesh = child as THREE.Mesh;
				if (mesh.geometry) {
					mesh.geometry.dispose();
				}
				if (mesh === capeMesh) {
					if (mesh.material) {
						const mat = mesh.material as THREE.Material;
						mat.dispose();
					}
					capeMesh = null;
				}
			}
		});
	}

	function buildMinecraftModel(isSlim: boolean) {
		if (playerGroup) {
			disposePlayerGroup(playerGroup);
			scene.remove(playerGroup);
		}
		playerGroup = new THREE.Group();

		const armWidth = isSlim ? 3 : 4;
		const armOffset = isSlim ? 5.5 : 6;

		// Head (8x8x8)
		const headGeom = new THREE.BoxGeometry(8, 8, 8);
		setBoxUV(headGeom, 0, 0, 8, 8, 8);
		const headMesh = new THREE.Mesh(headGeom, solidMaterial);
		headMesh.position.set(0, 20, 0);
		playerGroup.add(headMesh);

		// Hat Overlay (8.5x8.5x8.5)
		const hatGeom = new THREE.BoxGeometry(8.5, 8.5, 8.5);
		setBoxUV(hatGeom, 32, 0, 8, 8, 8);
		const hatMesh = new THREE.Mesh(hatGeom, overlayMaterial);
		hatMesh.position.set(0, 20, 0);
		playerGroup.add(hatMesh);

		// Torso (8x12x4)
		const torsoGeom = new THREE.BoxGeometry(8, 12, 4);
		setBoxUV(torsoGeom, 16, 16, 8, 12, 4);
		const torsoMesh = new THREE.Mesh(torsoGeom, solidMaterial);
		torsoMesh.position.set(0, 10, 0);
		playerGroup.add(torsoMesh);

		// Jacket Overlay (8.5x12.5x4.5)
		const jacketGeom = new THREE.BoxGeometry(8.5, 12.5, 4.5);
		setBoxUV(jacketGeom, 16, 32, 8, 12, 4);
		const jacketMesh = new THREE.Mesh(jacketGeom, overlayMaterial);
		jacketMesh.position.set(0, 10, 0);
		playerGroup.add(jacketMesh);

		// Right Arm
		const rArmGeom = new THREE.BoxGeometry(armWidth, 12, 4);
		setBoxUV(rArmGeom, 40, 16, armWidth, 12, 4);
		const rArmMesh = new THREE.Mesh(rArmGeom, solidMaterial);
		rArmMesh.position.set(-armOffset, 10, 0);
		playerGroup.add(rArmMesh);

		// Right Sleeve Overlay
		const rSleeveGeom = new THREE.BoxGeometry(armWidth + 0.5, 12.5, 4.5);
		setBoxUV(rSleeveGeom, 40, 32, armWidth, 12, 4);
		const rSleeveMesh = new THREE.Mesh(rSleeveGeom, overlayMaterial);
		rSleeveMesh.position.set(-armOffset, 10, 0);
		playerGroup.add(rSleeveMesh);

		// Left Arm
		const lArmGeom = new THREE.BoxGeometry(armWidth, 12, 4);
		setBoxUV(lArmGeom, 32, 48, armWidth, 12, 4);
		const lArmMesh = new THREE.Mesh(lArmGeom, solidMaterial);
		lArmMesh.position.set(armOffset, 10, 0);
		playerGroup.add(lArmMesh);

		// Left Sleeve Overlay
		const lSleeveGeom = new THREE.BoxGeometry(armWidth + 0.5, 12.5, 4.5);
		setBoxUV(lSleeveGeom, 48, 48, armWidth, 12, 4);
		const lSleeveMesh = new THREE.Mesh(lSleeveGeom, overlayMaterial);
		lSleeveMesh.position.set(armOffset, 10, 0);
		playerGroup.add(lSleeveMesh);

		// Right Leg (4x12x4)
		const rLegGeom = new THREE.BoxGeometry(4, 12, 4);
		setBoxUV(rLegGeom, 0, 16, 4, 12, 4);
		const rLegMesh = new THREE.Mesh(rLegGeom, solidMaterial);
		rLegMesh.position.set(-2, -2, 0);
		playerGroup.add(rLegMesh);

		// Right Pants Overlay
		const rPantsGeom = new THREE.BoxGeometry(4.5, 12.5, 4.5);
		setBoxUV(rPantsGeom, 0, 32, 4, 12, 4);
		const rPantsMesh = new THREE.Mesh(rPantsGeom, overlayMaterial);
		rPantsMesh.position.set(-2, -2, 0);
		playerGroup.add(rPantsMesh);

		// Left Leg (4x12x4)
		const lLegGeom = new THREE.BoxGeometry(4, 12, 4);
		setBoxUV(lLegGeom, 16, 48, 4, 12, 4);
		const lLegMesh = new THREE.Mesh(lLegGeom, solidMaterial);
		lLegMesh.position.set(2, -2, 0);
		playerGroup.add(lLegMesh);

		// Left Pants Overlay
		const lPantsGeom = new THREE.BoxGeometry(4.5, 12.5, 4.5);
		setBoxUV(lPantsGeom, 0, 48, 4, 12, 4);
		const lPantsMesh = new THREE.Mesh(lPantsGeom, overlayMaterial);
		lPantsMesh.position.set(2, -2, 0);
		playerGroup.add(lPantsMesh);

		// 3D Cape Mesh (attached behind torso at angle)
		updateCape();

		scene.add(playerGroup);
	}

	function updateCape() {
		if (capeMesh) {
			if (playerGroup) {
				playerGroup.remove(capeMesh);
			}
			if (capeMesh.geometry) {
				capeMesh.geometry.dispose();
			}
			if (capeMesh.material) {
				const mat = capeMesh.material as THREE.Material;
				mat.dispose();
			}
			capeMesh = null;
		}

		if (cape !== "none") {
			const capeGeom = new THREE.BoxGeometry(10, 16, 1);
			capeGeom.translate(0, -8, -0.5);
			setCapeUV(capeGeom);

			let capeTex: THREE.Texture;
			if (cape === "custom" && customCapeUrl) {
				const img = new Image();
				let normalizedUrl = customCapeUrl;
				if (normalizedUrl.startsWith("http://")) {
					normalizedUrl = normalizedUrl.replace("http://", "https://");
				}
				if (!normalizedUrl.startsWith("data:") && !normalizedUrl.startsWith("blob:")) {
					img.crossOrigin = "anonymous";
				}
				const tex = new THREE.Texture(img);
				tex.magFilter = THREE.NearestFilter;
				tex.minFilter = THREE.NearestFilter;
				tex.generateMipmaps = false;
				tex.colorSpace = THREE.SRGBColorSpace;
				img.onload = () => {
					tex.needsUpdate = true;
					if (capeMesh) {
						(capeMesh.material as THREE.MeshLambertMaterial).needsUpdate = true;
					}
				};
				img.src = normalizedUrl;
				capeTex = tex;
			} else {
				capeTex = createCapeTexture(cape);
			}

			const capeMat = new THREE.MeshLambertMaterial({
				map: capeTex,
				side: THREE.DoubleSide,
				transparent: true,
				alphaTest: 0.1
			});
			capeMesh = new THREE.Mesh(capeGeom, capeMat);
			capeMesh.position.set(0, 16, -2.35);
			capeMesh.rotation.x = 12 * (Math.PI / 180);
			playerGroup.add(capeMesh);
		}
	}

	let currentLoadedUrl = "";
	let loadSkinToken = 0;

	function loadSkin(url: string) {
		if (!url) return;
		let safeUrl = url;
		if (safeUrl.startsWith("http://")) {
			safeUrl = safeUrl.replace("http://", "https://");
		}
		if (safeUrl === currentLoadedUrl && currentTexture) return;

		const token = ++loadSkinToken;
		const img = new Image();
		if (!safeUrl.startsWith("data:") && !safeUrl.startsWith("blob:")) {
			img.crossOrigin = "anonymous";
		}
		img.onload = () => {
			if (token !== loadSkinToken) return;
			const canvas = document.createElement("canvas");
			canvas.width = 64;
			canvas.height = 64;
			const ctx = canvas.getContext("2d");
			if (!ctx) return;
			ctx.imageSmoothingEnabled = false;

			const isLegacy32 = img.height === img.width / 2 || img.height === 32;
			if (img.width !== 64 || (img.height !== 64 && !isLegacy32)) {
				ctx.drawImage(img, 0, 0, img.width, img.height, 0, 0, 64, isLegacy32 ? 32 : 64);
			} else {
				ctx.drawImage(img, 0, 0, 64, isLegacy32 ? 32 : 64);
			}

			if (isLegacy32) {
				ctx.clearRect(0, 32, 64, 32);
				ctx.imageSmoothingEnabled = false;

				const copyFlipped = (sx: number, sy: number, w: number, h: number, dx: number, dy: number) => {
					ctx.save();
					ctx.translate(dx + w, dy);
					ctx.scale(-1, 1);
					ctx.drawImage(canvas, sx, sy, w, h, 0, 0, w, h);
					ctx.restore();
				};

				// Left Leg from Right Leg (0, 16, 16, 16) -> (16, 48, 16, 16)
				copyFlipped(4, 16, 4, 4, 20, 48);
				copyFlipped(8, 16, 4, 4, 24, 48);
				copyFlipped(0, 20, 4, 12, 24, 52);
				copyFlipped(4, 20, 4, 12, 20, 52);
				copyFlipped(8, 20, 4, 12, 16, 52);
				copyFlipped(12, 20, 4, 12, 28, 52);

				// Left Arm from Right Arm (40, 16, 16, 16) -> (32, 48, 16, 16)
				copyFlipped(44, 16, 4, 4, 36, 48);
				copyFlipped(48, 16, 4, 4, 40, 48);
				copyFlipped(40, 20, 4, 12, 40, 52);
				copyFlipped(44, 20, 4, 12, 36, 52);
				copyFlipped(48, 20, 4, 12, 32, 52);
				copyFlipped(52, 20, 4, 12, 44, 52);
			}

			if (currentTexture) {
				currentTexture.dispose();
			}

			currentTexture = new THREE.CanvasTexture(canvas);
			currentTexture.magFilter = THREE.NearestFilter;
			currentTexture.minFilter = THREE.NearestFilter;
			currentTexture.generateMipmaps = false;
			currentTexture.colorSpace = THREE.SRGBColorSpace;

			if (solidMaterial) {
				solidMaterial.map = currentTexture;
				solidMaterial.needsUpdate = true;
			}
			if (overlayMaterial) {
				overlayMaterial.map = currentTexture;
				overlayMaterial.needsUpdate = true;
			}
			currentLoadedUrl = safeUrl;
		};
		img.onerror = () => {
			if (token !== loadSkinToken) return;
			if (currentLoadedUrl === "fallback") return;
			const canvas = createDefaultSteveCanvas();
			if (currentTexture) {
				currentTexture.dispose();
			}
			currentTexture = new THREE.CanvasTexture(canvas);
			currentTexture.magFilter = THREE.NearestFilter;
			currentTexture.minFilter = THREE.NearestFilter;
			currentTexture.generateMipmaps = false;
			currentTexture.colorSpace = THREE.SRGBColorSpace;
			if (solidMaterial) {
				solidMaterial.map = currentTexture;
				solidMaterial.needsUpdate = true;
			}
			if (overlayMaterial) {
				overlayMaterial.map = currentTexture;
				overlayMaterial.needsUpdate = true;
			}
			currentLoadedUrl = "fallback";
		};
		img.src = safeUrl;
	}

	function onPointerDown(e: MouseEvent) {
		isDragging = true;
		prevMouseX = e.clientX;
		prevMouseY = e.clientY;
		window.addEventListener("mousemove", handlePointerMove);
		window.addEventListener("mouseup", handlePointerUp);
	}

	function onWheel(e: WheelEvent) {
		e.preventDefault();
		zoom = Math.max(20, Math.min(65, zoom + e.deltaY * 0.03));
		updateCamera();
	}

	function updateCamera() {
		if (!camera) return;
		camera.position.x = zoom * Math.sin(yaw) * Math.cos(pitch);
		camera.position.y = 8 + zoom * Math.sin(pitch);
		camera.position.z = zoom * Math.cos(yaw) * Math.cos(pitch);
		camera.lookAt(0, 8, 0);
	}

	const MAX_CANVAS_WIDTH = 380;
	const MAX_CANVAS_HEIGHT = 440;

	onMount(() => {
		if (!containerEl) return;

		const width = Math.min(MAX_CANVAS_WIDTH, Math.floor(containerEl.clientWidth || 300));
		const height = Math.min(MAX_CANVAS_HEIGHT, Math.floor(containerEl.clientHeight || 340));

		scene = new THREE.Scene();
		camera = new THREE.PerspectiveCamera(40, width / height, 0.1, 1000);
		updateCamera();

		renderer = new THREE.WebGLRenderer({
			antialias: false,
			alpha: true,
			powerPreference: "default",
			depth: true,
			stencil: false,
			precision: "mediump"
		});
		renderer.setPixelRatio(Math.min(typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1, 1.5));
		renderer.setSize(width, height, false);
		renderer.shadowMap.enabled = false;
		renderer.domElement.style.position = "absolute";
		renderer.domElement.style.inset = "0";
		renderer.domElement.style.width = "100%";
		renderer.domElement.style.height = "100%";
		renderer.domElement.style.display = "block";
		renderer.domElement.style.pointerEvents = "none";
		containerEl.appendChild(renderer.domElement);

		const ambientLight = new THREE.AmbientLight(0xffffff, 0.85);
		scene.add(ambientLight);

		const dirLight = new THREE.DirectionalLight(0xffffff, 0.45);
		dirLight.position.set(15, 25, 20);
		scene.add(dirLight);

		solidMaterial = new THREE.MeshLambertMaterial({
			transparent: true,
			alphaTest: 0.05,
			side: THREE.FrontSide,
			depthWrite: true
		});

		overlayMaterial = new THREE.MeshLambertMaterial({
			transparent: true,
			alphaTest: 0.1,
			side: THREE.DoubleSide,
			depthWrite: true
		});

		buildMinecraftModel(slim);
		loadSkin(skinUrl);

		const observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					const wasVisible = isVisible;
					isVisible = entry.isIntersecting;
					if (active && isVisible && !wasVisible && animFrameId === null) {
						startLoop();
					} else if (!isVisible && animFrameId !== null) {
						stopLoop();
					}
				}
			},
			{ threshold: 0.05 }
		);
		observer.observe(containerEl);

		const handleVisibilityChange = () => {
			if (document.visibilityState === "hidden") {
				stopLoop();
			} else if (isVisible && active) {
				startLoop();
			}
		};
		document.addEventListener("visibilitychange", handleVisibilityChange);

		let lastW = width;
		let lastH = height;
		resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				const rawW = entry.contentRect.width;
				const rawH = entry.contentRect.height;
				const w = Math.min(MAX_CANVAS_WIDTH, Math.floor(rawW));
				const h = Math.min(MAX_CANVAS_HEIGHT, Math.floor(rawH));
				if (w > 0 && h > 0 && renderer && camera) {
					if (Math.abs(w - lastW) < 6 && Math.abs(h - lastH) < 6) continue;
					lastW = w;
					lastH = h;
					camera.aspect = w / h;
					camera.updateProjectionMatrix();
					renderer.setSize(w, h, false);
				}
			}
		});
		resizeObserver.observe(containerEl);

		startLoop();

		return () => {
			document.removeEventListener("visibilitychange", handleVisibilityChange);
			observer.disconnect();
			resizeObserver?.disconnect();
			stopLoop();
		};
	});

	let idleTime = 0;

	function animate() {
		if (!active || !isVisible || (typeof document !== "undefined" && document.visibilityState === "hidden") || !renderer) {
			animFrameId = null;
			return;
		}
		animFrameId = requestAnimationFrame(animate);

		if (autoRotate && !isDragging) {
			yaw += 0.007;
		}

		idleTime += 0.03;
		if (playerGroup) {
			const breathe = Math.sin(idleTime) * 0.015;
			playerGroup.position.y = breathe * 2;
			if (capeMesh) {
				capeMesh.rotation.x = (12 + Math.sin(idleTime * 1.5) * 2.5) * (Math.PI / 180);
			}
		}

		updateCamera();
		renderer.render(scene, camera);
	}

	function startLoop() {
		if (animFrameId === null && active && isVisible && renderer) {
			animFrameId = requestAnimationFrame(animate);
		}
	}

	function stopLoop() {
		if (animFrameId !== null) {
			cancelAnimationFrame(animFrameId);
			animFrameId = null;
		}
	}

	function handlePointerMove(e: MouseEvent) {
		if (!isDragging) return;
		const dx = e.clientX - prevMouseX;
		const dy = e.clientY - prevMouseY;

		yaw += dx * 0.012;
		pitch = Math.max(-0.6, Math.min(0.6, pitch + dy * 0.008));

		prevMouseX = e.clientX;
		prevMouseY = e.clientY;
	}

	function handlePointerUp() {
		isDragging = false;
	}

	onDestroy(() => {
		window.removeEventListener("mousemove", handlePointerMove);
		window.removeEventListener("mouseup", handlePointerUp);

		if (animFrameId !== null) {
			cancelAnimationFrame(animFrameId);
			animFrameId = null;
		}
		if (resizeObserver) {
			resizeObserver.disconnect();
			resizeObserver = null;
		}
		if (playerGroup) {
			disposePlayerGroup(playerGroup);
			if (scene) scene.remove(playerGroup);
			playerGroup = null as unknown as THREE.Group;
		}
		if (capeMesh) {
			if (capeMesh.geometry) capeMesh.geometry.dispose();
			if (capeMesh.material) {
				const mat = capeMesh.material as THREE.Material;
				mat.dispose();
			}
			capeMesh = null;
		}
		for (const tex of capeTextureCache.values()) {
			tex.dispose();
		}
		capeTextureCache.clear();
		if (currentTexture) {
			currentTexture.dispose();
			currentTexture = null;
		}
		if (solidMaterial) {
			solidMaterial.dispose();
		}
		if (overlayMaterial) {
			overlayMaterial.dispose();
		}
		if (renderer) {
			if (renderer.domElement && renderer.domElement.parentNode) {
				renderer.domElement.parentNode.removeChild(renderer.domElement);
			}
			renderer.dispose();
			renderer.forceContextLoss();
		}
	});

	let currentSlim: boolean | null = null;
	let currentCape: CapeType | null = null;

	$effect(() => {
		const url = skinUrl;
		if (url && solidMaterial) {
			loadSkin(url);
		}
	});

	$effect(() => {
		const isSlim = slim;
		if (playerGroup) {
			if (currentSlim === null) {
				currentSlim = isSlim;
			} else if (isSlim !== currentSlim) {
				currentSlim = isSlim;
				buildMinecraftModel(isSlim);
			}
		}
	});

	let currentCustomCapeUrl: string | null = null;

	$effect(() => {
		const c = cape;
		const customUrl = customCapeUrl || "";
		if (playerGroup) {
			if (c !== currentCape || customUrl !== currentCustomCapeUrl) {
				currentCape = c;
				currentCustomCapeUrl = customUrl;
				updateCape();
			}
		}
	});

	$effect(() => {
		const isAct = active;
		if (isAct) {
			startLoop();
		} else {
			stopLoop();
		}
	});

	export function setAngle(deg: number) {
		yaw = deg * (Math.PI / 180);
		pitch = 4 * (Math.PI / 180);
		updateCamera();
	}

	export function zoomIn() {
		zoom = Math.max(20, zoom - 4);
		updateCamera();
	}

	export function zoomOut() {
		zoom = Math.min(65, zoom + 4);
		updateCamera();
	}

	export function resetView() {
		zoom = 36;
		yaw = 20 * (Math.PI / 180);
		pitch = 4 * (Math.PI / 180);
		updateCamera();
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div 
	bind:this={containerEl}
	class="relative w-full h-full cursor-grab active:cursor-grabbing select-none overflow-hidden {className}"
	onmousedown={onPointerDown}
	onwheel={onWheel}
	role="region"
	aria-label="Visualizador 3D de Skin"
>
	<div class="absolute bottom-2 left-1/2 -translate-x-1/2 w-32 h-6 bg-black/40 rounded-full blur-sm pointer-events-none"></div>
</div>
