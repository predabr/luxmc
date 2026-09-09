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
	let yaw = 25 * (Math.PI / 180);
	let pitch = 10 * (Math.PI / 180);
	let zoom = 58;

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

	function createCapeTexture(type: string): THREE.CanvasTexture {
		const canvas = document.createElement("canvas");
		canvas.width = 64;
		canvas.height = 32;
		const ctx = canvas.getContext("2d")!;

		if (type === "migrator") {
			// Red gradient with golden star
			const grad = ctx.createLinearGradient(0, 0, 0, 32);
			grad.addColorStop(0, "#881337");
			grad.addColorStop(1, "#4c0519");
			ctx.fillStyle = grad;
			ctx.fillRect(0, 0, 64, 32);
			ctx.fillStyle = "#e2b86b";
			ctx.fillRect(28, 12, 8, 8);
		} else if (type === "optifine") {
			// Red with white OF
			ctx.fillStyle = "#dc2626";
			ctx.fillRect(0, 0, 64, 32);
			ctx.fillStyle = "#ffffff";
			ctx.font = "bold 16px sans-serif";
			ctx.textAlign = "center";
			ctx.fillText("OF", 32, 22);
		} else {
			// Classic Mojang
			ctx.fillStyle = "#991b1b";
			ctx.fillRect(0, 0, 64, 32);
			ctx.fillStyle = "#ffffff";
			ctx.fillRect(26, 10, 12, 12);
		}

		const tex = new THREE.CanvasTexture(canvas);
		tex.magFilter = THREE.NearestFilter;
		tex.minFilter = THREE.NearestFilter;
		return tex;
	}

	function buildMinecraftModel(isSlim: boolean) {
		if (playerGroup) {
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
			playerGroup.remove(capeMesh);
			capeMesh.geometry.dispose();
			capeMesh = null;
		}

		if (cape !== "none") {
			const capeGeom = new THREE.BoxGeometry(10, 16, 1);
			const capeTex = createCapeTexture(cape);
			const capeMat = new THREE.MeshStandardMaterial({
				map: capeTex,
				roughness: 0.8,
				metalness: 0.1
			});
			capeMesh = new THREE.Mesh(capeGeom, capeMat);
			// Position on upper back with dynamic slight wind tilt
			capeMesh.position.set(0, 8, -2.8);
			capeMesh.rotation.x = -14 * (Math.PI / 180);
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
			const ctx = canvas.getContext("2d")!;
			ctx.drawImage(img, 0, 0);

			// If it's an old 64x32 skin, duplicate limbs to 64x64 format
			if (img.height === 32) {
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
		zoom = Math.max(28, Math.min(75, zoom + e.deltaY * 0.04));
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

		renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
		renderer.setSize(width, height);
		renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
		renderer.shadowMap.enabled = false;
		containerEl.appendChild(renderer.domElement);

		// Lights
		const ambientLight = new THREE.AmbientLight(0xffffff, 1.4);
		scene.add(ambientLight);

		const frontLight = new THREE.DirectionalLight(0xfff7ed, 1.1);
		frontLight.position.set(20, 40, 30);
		scene.add(frontLight);

		const backLight = new THREE.DirectionalLight(0x93c5fd, 0.6);
		backLight.position.set(-20, -10, -30);
		scene.add(backLight);

		// Materials
		solidMaterial = new THREE.MeshStandardMaterial({
			roughness: 0.85,
			metalness: 0.05
		});

		overlayMaterial = new THREE.MeshStandardMaterial({
			roughness: 0.85,
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
				yaw += 0.008;
			}

			// Gentle breathing idle motion
			idleTime += 0.03;
			if (playerGroup) {
				const breathe = Math.sin(idleTime) * 0.015;
				playerGroup.position.y = breathe * 2;
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
		}
		if (renderer && renderer.domElement) {
			renderer.domElement.remove();
			renderer.dispose();
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
		pitch = 0.05;
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
