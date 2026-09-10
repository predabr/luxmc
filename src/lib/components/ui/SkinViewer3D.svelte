<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import * as THREE from "three";

	type Props = {
		skinUrl?: string;
		cape?: "none" | "migrator" | "optifine" | "mojang";
		slim?: boolean;
		autoRotate?: boolean;
		className?: string;
	};

	let {
		skinUrl = "https://minotar.net/skin/Steve",
		cape = "none",
		slim = false,
		autoRotate = true,
		className = ""
	}: Props = $props();

	let containerEl: HTMLDivElement | null = $state(null);
	let animFrameId: number | null = null;
	let scene: THREE.Scene;
	let camera: THREE.PerspectiveCamera;
	let renderer: THREE.WebGLRenderer;
	let playerGroup: THREE.Group;
	let capeMesh: THREE.Mesh | null = null;

	// Texture and materials
	let currentTexture: THREE.CanvasTexture | THREE.Texture | null = null;
	let solidMaterial: THREE.MeshStandardMaterial;
	let overlayMaterial: THREE.MeshStandardMaterial;

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

		function setFace(faceIndex: number, u1: number, v1: number, u2: number, v2: number, flipX = false) {
			const i = faceIndex * 4;
			const leftU = flipX ? u2 : u1;
			const rightU = flipX ? u1 : u2;
			uv.setXY(i + 0, leftU / tw, 1 - v1 / th);
			uv.setXY(i + 1, rightU / tw, 1 - v1 / th);
			uv.setXY(i + 2, leftU / tw, 1 - v2 / th);
			uv.setXY(i + 3, rightU / tw, 1 - v2 / th);
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
		// 4: Front (+Z, inner face facing player): (1, 1) to (11, 17)
		setFace(4, 1, 1, 11, 17);
		// 5: Back (-Z, outer face facing viewer): (12, 1) to (22, 17) with flipX so text reads correctly
		setFace(5, 12, 1, 22, 17, true);

		uv.needsUpdate = true;
	}

	function createCapeTexture(type: string): THREE.CanvasTexture {
		const canvas = document.createElement("canvas");
		canvas.width = 64;
		canvas.height = 32;
		const ctx = canvas.getContext("2d")!;
		ctx.imageSmoothingEnabled = false;

		if (type === "migrator") {
			// Rich deep wine/burgundy base
			ctx.fillStyle = "#5c0818";
			ctx.fillRect(0, 0, 64, 32);

			// Shading on inner face
			ctx.fillStyle = "#40040f";
			ctx.fillRect(1, 1, 10, 16);

			// Back face outer: (12, 1) to (22, 17)
			ctx.fillStyle = "#690a1c";
			ctx.fillRect(12, 1, 10, 16);

			// Golden Migrator Emblem:
			const gold = "#d8a646";
			const lightGold = "#fae596";
			const darkGold = "#9d7426";

			// Golden border trim on bottom
			ctx.fillStyle = darkGold;
			ctx.fillRect(12, 16, 10, 1);
			ctx.fillStyle = gold;
			ctx.fillRect(13, 15, 8, 1);

			// Migrator Crest (Roman I / Pillar)
			// Top crossbar
			ctx.fillStyle = gold;
			ctx.fillRect(14, 4, 6, 2);
			ctx.fillStyle = lightGold;
			ctx.fillRect(15, 4, 4, 1);

			// Center pillar
			ctx.fillStyle = gold;
			ctx.fillRect(16, 6, 2, 6);
			ctx.fillStyle = lightGold;
			ctx.fillRect(16, 7, 1, 4);

			// Bottom crossbar
			ctx.fillStyle = gold;
			ctx.fillRect(14, 12, 6, 2);
			ctx.fillStyle = darkGold;
			ctx.fillRect(14, 13, 6, 1);
		} else if (type === "optifine") {
			// Vivid OptiFine Red
			ctx.fillStyle = "#b51a1a";
			ctx.fillRect(0, 0, 64, 32);

			// Inner face
			ctx.fillStyle = "#8a1010";
			ctx.fillRect(1, 1, 10, 16);

			// Outer face
			ctx.fillStyle = "#c92020";
			ctx.fillRect(12, 1, 10, 16);

			// White OF text
			ctx.fillStyle = "#ffffff";
			// Letter 'O' (14..16, 6..11)
			ctx.fillRect(14, 6, 3, 6);
			ctx.fillStyle = "#c92020";
			ctx.fillRect(15, 7, 1, 4);

			// Letter 'F' (18..20, 6..11)
			ctx.fillStyle = "#ffffff";
			ctx.fillRect(18, 6, 1, 6);
			ctx.fillRect(18, 6, 3, 1);
			ctx.fillRect(18, 8, 2, 1);
		} else {
			// Classic Mojang Crimson
			ctx.fillStyle = "#8b0e17";
			ctx.fillRect(0, 0, 64, 32);

			// Inner face
			ctx.fillStyle = "#63070e";
			ctx.fillRect(1, 1, 10, 16);

			// Outer face
			ctx.fillStyle = "#a8121d";
			ctx.fillRect(12, 1, 10, 16);

			// Mojang Studio white crest
			ctx.fillStyle = "#ffffff";
			ctx.fillRect(15, 7, 4, 4);
			ctx.fillStyle = "#a8121d";
			ctx.fillRect(16, 8, 2, 2);
		}

		const tex = new THREE.CanvasTexture(canvas);
		tex.magFilter = THREE.NearestFilter;
		tex.minFilter = THREE.NearestFilter;
		tex.generateMipmaps = false;
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
						const mat = mesh.material as THREE.MeshStandardMaterial;
						if (mat.map) mat.map.dispose();
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
				const mat = capeMesh.material as THREE.MeshStandardMaterial;
				if (mat.map) mat.map.dispose();
				mat.dispose();
			}
			capeMesh = null;
		}

		if (cape !== "none") {
			const capeGeom = new THREE.BoxGeometry(10, 16, 1);
			setCapeUV(capeGeom);

			const capeTex = createCapeTexture(cape);
			const capeMat = new THREE.MeshStandardMaterial({
				map: capeTex,
				roughness: 0.65,
				metalness: 0.05
			});
			capeMesh = new THREE.Mesh(capeGeom, capeMat);
			capeMesh.position.set(0, 8, -2.6);
			capeMesh.rotation.x = -15 * (Math.PI / 180);
			playerGroup.add(capeMesh);
		}
	}

	function loadSkin(url: string) {
		const img = new Image();
		img.crossOrigin = "anonymous";
		img.onload = () => {
			const canvas = document.createElement("canvas");
			canvas.width = 64;
			canvas.height = 64;
			const ctx = canvas.getContext("2d", { willReadFrequently: true })!;
			ctx.imageSmoothingEnabled = false;
			ctx.drawImage(img, 0, 0);

			// If it's an old 64x32 skin, duplicate limbs to 64x64 format
			if (img.height === 32) {
				ctx.imageSmoothingEnabled = false;
				// Copy right arm to left arm
				ctx.drawImage(canvas, 40, 16, 16, 16, 32, 48, 16, 16);
				// Copy right leg to left leg
				ctx.drawImage(canvas, 0, 16, 16, 16, 16, 48, 16, 16);
			}

			if (currentTexture) {
				currentTexture.dispose();
			}

			currentTexture = new THREE.CanvasTexture(canvas);
			currentTexture.magFilter = THREE.NearestFilter;
			currentTexture.minFilter = THREE.NearestFilter;
			currentTexture.generateMipmaps = false;
			currentTexture.colorSpace = THREE.SRGBColorSpace;

			solidMaterial.map = currentTexture;
			solidMaterial.needsUpdate = true;

			overlayMaterial.map = currentTexture;
			overlayMaterial.needsUpdate = true;
		};
		img.onerror = () => {
			// Fallback to minotar default steve if custom url fails
			if (!url.includes("Steve")) {
				loadSkin("https://minotar.net/skin/Steve");
			}
		};
		img.src = url;
	}

	function onPointerDown(e: MouseEvent) {
		isDragging = true;
		prevMouseX = e.clientX;
		prevMouseY = e.clientY;
	}

	function onPointerMove(e: MouseEvent) {
		if (!isDragging) return;
		const dx = e.clientX - prevMouseX;
		const dy = e.clientY - prevMouseY;

		yaw += dx * 0.012;
		pitch = Math.max(-0.6, Math.min(0.6, pitch + dy * 0.008));

		prevMouseX = e.clientX;
		prevMouseY = e.clientY;
	}

	function onPointerUp() {
		isDragging = false;
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

	onMount(() => {
		if (!containerEl) return;

		const width = containerEl.clientWidth || 300;
		const height = containerEl.clientHeight || 340;

		scene = new THREE.Scene();
		camera = new THREE.PerspectiveCamera(40, width / height, 0.1, 1000);
		updateCamera();

		renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: "high-performance" });
		renderer.setSize(width, height);
		renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
		renderer.toneMapping = THREE.ACESFilmicToneMapping;
		renderer.toneMappingExposure = 1.15;
		renderer.shadowMap.enabled = false;
		containerEl.appendChild(renderer.domElement);

		// Studio 3-point lighting setup
		const ambientLight = new THREE.AmbientLight(0xffffff, 0.75);
		scene.add(ambientLight);

		const frontLight = new THREE.DirectionalLight(0xfff7ed, 1.5);
		frontLight.position.set(20, 35, 30);
		scene.add(frontLight);

		const fillLight = new THREE.DirectionalLight(0xe0e7ff, 0.8);
		fillLight.position.set(-20, 15, 20);
		scene.add(fillLight);

		const backLight = new THREE.DirectionalLight(0xfef3c7, 1.2);
		backLight.position.set(0, 25, -30);
		scene.add(backLight);

		// Materials
		solidMaterial = new THREE.MeshStandardMaterial({
			roughness: 0.65,
			metalness: 0.05
		});

		overlayMaterial = new THREE.MeshStandardMaterial({
			roughness: 0.65,
			metalness: 0.05,
			transparent: true,
			opacity: 1,
			alphaTest: 0.5,
			side: THREE.DoubleSide
		});

		buildMinecraftModel(slim);
		loadSkin(skinUrl);

		// Animation loop
		let idleTime = 0;
		const animate = () => {
			animFrameId = requestAnimationFrame(animate);

			if (autoRotate && !isDragging) {
				yaw += 0.007;
			}

			// Gentle breathing idle motion and cape flutter
			idleTime += 0.03;
			if (playerGroup) {
				const breathe = Math.sin(idleTime) * 0.015;
				playerGroup.position.y = breathe * 2;
				if (capeMesh) {
					capeMesh.rotation.x = (-15 + Math.sin(idleTime * 1.5) * 2.5) * (Math.PI / 180);
				}
			}

			updateCamera();
			renderer.render(scene, camera);
		};
		animate();

		const resizeObserver = new ResizeObserver((entries) => {
			for (const entry of entries) {
				const w = entry.contentRect.width;
				const h = entry.contentRect.height;
				if (w > 0 && h > 0 && renderer && camera) {
					camera.aspect = w / h;
					camera.updateProjectionMatrix();
					renderer.setSize(w, h);
				}
			}
		});
		resizeObserver.observe(containerEl);

		return () => {
			resizeObserver.disconnect();
		};
	});

	onDestroy(() => {
		if (animFrameId !== null) {
			cancelAnimationFrame(animFrameId);
			animFrameId = null;
		}
		if (playerGroup) {
			disposePlayerGroup(playerGroup);
			if (scene) scene.remove(playerGroup);
			playerGroup = null as unknown as THREE.Group;
		}
		if (capeMesh) {
			if (capeMesh.geometry) capeMesh.geometry.dispose();
			if (capeMesh.material) {
				const mat = capeMesh.material as THREE.MeshStandardMaterial;
				if (mat.map) mat.map.dispose();
				mat.dispose();
			}
			capeMesh = null;
		}
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

	$effect(() => {
		const url = skinUrl;
		if (url && solidMaterial) {
			loadSkin(url);
		}
	});

	$effect(() => {
		const isSlim = slim;
		if (playerGroup) {
			buildMinecraftModel(isSlim);
		}
	});

	$effect(() => {
		const c = cape;
		if (playerGroup) {
			updateCape();
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

<svelte:window onmousemove={onPointerMove} onmouseup={onPointerUp} />

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div 
	bind:this={containerEl}
	class="relative w-full h-full cursor-grab active:cursor-grabbing select-none {className}"
	onmousedown={onPointerDown}
	onwheel={onWheel}
	role="region"
	aria-label="Visualizador 3D de Skin"
>
	<!-- Subtle Floor Pedestal Ring -->
	<div class="absolute bottom-2 left-1/2 -translate-x-1/2 w-32 h-6 bg-black/40 rounded-full blur-sm pointer-events-none"></div>
</div>
