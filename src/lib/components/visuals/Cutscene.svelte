<script lang="ts">
	import { onMount } from "svelte";
	import gsap from "gsap";
	import * as THREE from "three";
	import { Volume2, VolumeX } from "lucide-svelte";
	import { settings } from "$lib/stores/settings.svelte";
	
	let { onComplete } = $props<{ onComplete: () => void }>();
	
	let container: HTMLElement;
	let canvasContainer: HTMLDivElement;
	let flashOverlay: HTMLElement;
	let shockwave1: HTMLElement;
	let shockwave2: HTMLElement;
	let chromeGleam: HTMLElement;
	let titleText: HTMLElement;
	let subtitleText: HTMLElement;
	let techBadges: HTMLElement;
	let letterboxTop: HTMLElement;
	let letterboxBottom: HTMLElement;
	let isMuted = $state(false);

	let reqId: number | null = null;
	let tl: gsap.core.Timeline | null = null;
	let audioCtx: AudioContext | null = null;

	function initAudio() {
		try {
			const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
			if (AudioContextClass) {
				audioCtx = new AudioContextClass();
			}
		} catch {
			audioCtx = null;
		}
	}

	function playChargeSound() {
		if (!audioCtx || isMuted) return;
		try {
			if (audioCtx.state === "suspended") void audioCtx.resume();
			const now = audioCtx.currentTime;
			const osc = audioCtx.createOscillator();
			const gain = audioCtx.createGain();
			osc.type = "sine";
			osc.frequency.setValueAtTime(55, now);
			osc.frequency.exponentialRampToValueAtTime(280, now + 0.38);
			gain.gain.setValueAtTime(0.01, now);
			gain.gain.linearRampToValueAtTime(0.18, now + 0.3);
			gain.gain.linearRampToValueAtTime(0, now + 0.4);
			osc.connect(gain);
			gain.connect(audioCtx.destination);
			osc.start(now);
			osc.stop(now + 0.4);
		} catch {}
	}

	function playShatterSound() {
		if (!audioCtx || isMuted) return;
		try {
			if (audioCtx.state === "suspended") void audioCtx.resume();
			const now = audioCtx.currentTime;
			
			const sub = audioCtx.createOscillator();
			const subGain = audioCtx.createGain();
			sub.type = "triangle";
			sub.frequency.setValueAtTime(140, now);
			sub.frequency.exponentialRampToValueAtTime(32, now + 0.45);
			subGain.gain.setValueAtTime(0.35, now);
			subGain.gain.exponentialRampToValueAtTime(0.001, now + 0.5);
			sub.connect(subGain);
			subGain.connect(audioCtx.destination);
			sub.start(now);
			sub.stop(now + 0.5);

			const bufferSize = Math.floor(audioCtx.sampleRate * 0.35);
			const buffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
			const data = buffer.getChannelData(0);
			for (let i = 0; i < bufferSize; i++) {
				data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (audioCtx.sampleRate * 0.05));
			}
			const noise = audioCtx.createBufferSource();
			noise.buffer = buffer;
			const filter = audioCtx.createBiquadFilter();
			filter.type = "highpass";
			filter.frequency.setValueAtTime(2800, now);
			const noiseGain = audioCtx.createGain();
			noiseGain.gain.setValueAtTime(0.28, now);
			noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.35);
			noise.connect(filter);
			filter.connect(noiseGain);
			noiseGain.connect(audioCtx.destination);
			noise.start(now);

			const chime = audioCtx.createOscillator();
			const chimeGain = audioCtx.createGain();
			chime.type = "sine";
			chime.frequency.setValueAtTime(1560, now);
			chimeGain.gain.setValueAtTime(0.12, now);
			chimeGain.gain.exponentialRampToValueAtTime(0.001, now + 0.4);
			chime.connect(chimeGain);
			chimeGain.connect(audioCtx.destination);
			chime.start(now);
			chime.stop(now + 0.4);
		} catch {}
	}

	function playVortexSound() {
		if (!audioCtx || isMuted) return;
		try {
			if (audioCtx.state === "suspended") void audioCtx.resume();
			const now = audioCtx.currentTime;
			const osc = audioCtx.createOscillator();
			const gain = audioCtx.createGain();
			osc.type = "sine";
			osc.frequency.setValueAtTime(70, now);
			osc.frequency.exponentialRampToValueAtTime(480, now + 1.05);
			gain.gain.setValueAtTime(0.01, now);
			gain.gain.linearRampToValueAtTime(0.22, now + 0.85);
			gain.gain.linearRampToValueAtTime(0.01, now + 1.1);
			osc.connect(gain);
			gain.connect(audioCtx.destination);
			osc.start(now);
			osc.stop(now + 1.15);
		} catch {}
	}

	function playLockSound() {
		if (!audioCtx || isMuted) return;
		try {
			if (audioCtx.state === "suspended") void audioCtx.resume();
			const now = audioCtx.currentTime;
			
			const snap = audioCtx.createOscillator();
			const snapGain = audioCtx.createGain();
			snap.type = "triangle";
			snap.frequency.setValueAtTime(520, now);
			snap.frequency.exponentialRampToValueAtTime(95, now + 0.14);
			snapGain.gain.setValueAtTime(0.3, now);
			snapGain.gain.exponentialRampToValueAtTime(0.001, now + 0.15);
			snap.connect(snapGain);
			snapGain.connect(audioCtx.destination);
			snap.start(now);
			snap.stop(now + 0.15);

			const b1 = audioCtx.createOscillator();
			const b2 = audioCtx.createOscillator();
			const bg = audioCtx.createGain();
			b1.type = "sine";
			b2.type = "sine";
			b1.frequency.setValueAtTime(880, now);
			b2.frequency.setValueAtTime(1760, now);
			bg.gain.setValueAtTime(0.22, now);
			bg.gain.exponentialRampToValueAtTime(0.001, now + 0.75);
			b1.connect(bg);
			b2.connect(bg);
			bg.connect(audioCtx.destination);
			b1.start(now);
			b2.start(now);
			b1.stop(now + 0.75);
			b2.stop(now + 0.75);
		} catch {}
	}

	onMount(() => {
		initAudio();

		const width = window.innerWidth;
		const height = window.innerHeight;

		const scene = new THREE.Scene();
		scene.fog = new THREE.FogExp2(0x050608, 0.038);

		const camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 100);
		camera.position.set(0, 0, 7.8);

		const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: "high-performance" });
		renderer.setSize(width, height);
		renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
		renderer.toneMapping = THREE.ACESFilmicToneMapping;
		renderer.toneMappingExposure = 1.35;
		canvasContainer.appendChild(renderer.domElement);

		const ambientLight = new THREE.AmbientLight(0xfff3db, 1.3);
		scene.add(ambientLight);

		const mainLight = new THREE.DirectionalLight(0xffe29a, 2.6);
		mainLight.position.set(5, 7, 6);
		scene.add(mainLight);

		const rimLight = new THREE.DirectionalLight(0x38bdf8, 2.2);
		rimLight.position.set(-6, -4, -3);
		scene.add(rimLight);

		const corePointLight = new THREE.PointLight(0xffd700, 0, 15);
		corePointLight.position.set(0, 0, 0.5);
		scene.add(corePointLight);

		const logoGroup = new THREE.Group();
		scene.add(logoGroup);

		interface FluidShard {
			mesh: THREE.Mesh;
			originPos: THREE.Vector3;
			explodedPos: THREE.Vector3;
			originRot: THREE.Euler;
			explodedRot: THREE.Euler;
			dist: number;
		}

		const shards: FluidShard[] = [];
		let intactMesh: THREE.Mesh | null = null;

		// 3D Thick Glowing Plasma Beams
		const BEAM_COUNT = 24;
		const beamMeshes: THREE.Mesh[] = [];
		const beamGeom = new THREE.CylinderGeometry(0.1, 0.1, 1, 8);
		beamGeom.translate(0, 0.5, 0);

		const beamMat = new THREE.MeshStandardMaterial({
			color: 0xffea78,
			emissive: 0xffaa00,
			emissiveIntensity: 3.5,
			roughness: 0.1,
			metalness: 0.8,
			transparent: true,
			opacity: 0,
			blending: THREE.AdditiveBlending,
			side: THREE.DoubleSide
		});

		const beamGroup = new THREE.Group();
		scene.add(beamGroup);

		for (let i = 0; i < BEAM_COUNT; i++) {
			const bMesh = new THREE.Mesh(beamGeom, beamMat.clone());
			bMesh.visible = false;
			beamGroup.add(bMesh);
			beamMeshes.push(bMesh);
		}

		let beamGlobalOpacity = 0;

		const img = new Image();
		img.src = "/logo.png";
		img.onload = () => {
			const tex = new THREE.Texture(img);
			tex.needsUpdate = true;
			tex.colorSpace = THREE.SRGBColorSpace;
			tex.minFilter = THREE.LinearFilter;
			tex.magFilter = THREE.LinearFilter;
			buildLogoShards(tex, img);
			startCutsceneTimeline();
		};
		if (img.complete) {
			const tex = new THREE.Texture(img);
			tex.needsUpdate = true;
			tex.colorSpace = THREE.SRGBColorSpace;
			tex.minFilter = THREE.LinearFilter;
			tex.magFilter = THREE.LinearFilter;
			buildLogoShards(tex, img);
			startCutsceneTimeline();
		}

		function buildLogoShards(tex: THREE.Texture, imageSource: HTMLImageElement) {
			const frontMat = new THREE.MeshStandardMaterial({
				map: tex,
				color: 0xffffff,
				roughness: 0.12,
				metalness: 0.35,
				emissive: 0xffffff,
				emissiveMap: tex,
				emissiveIntensity: 0.5,
				side: THREE.DoubleSide,
				transparent: true,
				alphaTest: 0.02
			});

			const sideMat = new THREE.MeshStandardMaterial({
				color: 0xdfab39,
				roughness: 0.16,
				metalness: 0.95,
				side: THREE.DoubleSide
			});

			
			const ROWS = 6;
			const COLS = 6;
			const size = 4.4;
			const thickness = 0.12;

			const intactGeom = new THREE.PlaneGeometry(size, size);
			const intactMat = new THREE.MeshStandardMaterial({
				map: tex,
				transparent: true,
				roughness: 0.1,
				metalness: 0.2,
				emissive: 0xffffff,
				emissiveMap: tex,
				emissiveIntensity: 0.5,
				side: THREE.DoubleSide,
				alphaTest: 0.01
			});
			intactMesh = new THREE.Mesh(intactGeom, intactMat);
			intactMesh.position.set(0, 0, 0.02);
			intactMesh.visible = true;
			logoGroup.add(intactMesh);

			const canvas = document.createElement("canvas");
			canvas.width = 128;
			canvas.height = 128;
			const ctx = canvas.getContext("2d");
			let imgData: Uint8ClampedArray | null = null;
			if (ctx) {
				try {
					ctx.drawImage(imageSource, 0, 0, 128, 128);
					imgData = ctx.getImageData(0, 0, 128, 128).data;
				} catch {}
			}

			function getAlpha(normX: number, normY: number): number {
				if (!imgData) return 255;
				const px = Math.floor(Math.max(0, Math.min(127, normX * 127)));
				const py = Math.floor(Math.max(0, Math.min(127, (1 - normY) * 127)));
				return imgData[(py * 128 + px) * 4 + 3];
			}

			const grid: { x: number; y: number }[][] = [];
			for (let r = 0; r <= ROWS; r++) {
				grid[r] = [];
				for (let c = 0; c <= COLS; c++) {
					let gx = -width / 2 + (c / COLS) * width;
					let gy = -height / 2 + (r / ROWS) * height;
					if (r > 0 && r < ROWS && c > 0 && c < COLS) {
						gx += (Math.random() - 0.5) * (width / COLS) * 0.72;
						gy += (Math.random() - 0.5) * (height / ROWS) * 0.72;
					}
					grid[r][c] = { x: gx, y: gy };
				}
			}

			for (let r = 0; r < ROWS; r++) {
				for (let c = 0; c < COLS; c++) {
					const tl = grid[r][c];
					const tr = grid[r][c + 1];
					const br = grid[r + 1][c + 1];
					const bl = grid[r + 1][c];
					const center = {
						x: (tl.x + tr.x + br.x + bl.x) / 4 + (Math.random() - 0.5) * (width / COLS) * 0.35,
						y: (tl.y + tr.y + br.y + bl.y) / 4 + (Math.random() - 0.5) * (height / ROWS) * 0.35
					};

					const triSets = [
						[tl, tr, center],
						[tr, br, center],
						[br, bl, center],
						[bl, tl, center]
					];

					for (const [pA, pB, pC] of triSets) {
						const cx = (pA.x + pB.x + pC.x) / 3;
						const cy = (pA.y + pB.y + pC.y) / 3;

						const nA = { x: (pA.x + width / 2) / width, y: (pA.y + height / 2) / height };
						const nB = { x: (pB.x + width / 2) / width, y: (pB.y + height / 2) / height };
						const nC = { x: (pC.x + width / 2) / width, y: (pC.y + height / 2) / height };
						const nCenter = { x: (cx + width / 2) / width, y: (cy + height / 2) / height };

						const avgAlpha = (getAlpha(nA.x, nA.y) + getAlpha(nB.x, nB.y) + getAlpha(nC.x, nC.y) + getAlpha(nCenter.x, nCenter.y)) / 4;
						if (avgAlpha < 50) continue;

						let p1 = pA, p2 = pB, p3 = pC;
						const cross = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);
						if (cross < 0) {
							p2 = pC;
							p3 = pB;
						}

						const a = { x: p1.x - cx, y: p1.y - cy };
						const b = { x: p2.x - cx, y: p2.y - cy };
						const c = { x: p3.x - cx, y: p3.y - cy };
						const zf = thickness / 2;
						const zb = -thickness / 2;

						const positions = [
							// Front
							a.x, a.y, zf,  b.x, b.y, zf,  c.x, c.y, zf,
							// Back
							a.x, a.y, zb,  c.x, c.y, zb,  b.x, b.y, zb,
							// Side A-B
							a.x, a.y, zf,  a.x, a.y, zb,  b.x, b.y, zb,
							a.x, a.y, zf,  b.x, b.y, zb,  b.x, b.y, zf,
							// Side B-C
							b.x, b.y, zf,  b.x, b.y, zb,  c.x, c.y, zb,
							b.x, b.y, zf,  c.x, c.y, zb,  c.x, c.y, zf,
							// Side C-A
							c.x, c.y, zf,  c.x, c.y, zb,  a.x, a.y, zb,
							c.x, c.y, zf,  a.x, a.y, zb,  a.x, a.y, zf,
						];

						const uvs = [
							// Front UVs
							(p1.x + width / 2) / width, (p1.y + height / 2) / height,
							(p2.x + width / 2) / width, (p2.y + height / 2) / height,
							(p3.x + width / 2) / width, (p3.y + height / 2) / height,
							// Back UVs (mirrored)
							(p1.x + width / 2) / width, (p1.y + height / 2) / height,
							(p3.x + width / 2) / width, (p3.y + height / 2) / height,
							(p2.x + width / 2) / width, (p2.y + height / 2) / height,
							// Side UVs
							0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
							0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
							0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
						];

						const geom = new THREE.BufferGeometry();
						geom.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
						geom.setAttribute("uv", new THREE.Float32BufferAttribute(uvs, 2));
						geom.addGroup(0, 6, 0);
						geom.addGroup(6, 18, 1);
						geom.computeVertexNormals();

						const mesh = new THREE.Mesh(geom, [frontMat, sideMat]);
						mesh.position.set(cx, cy, 0);
						mesh.visible = false;

						const dist = Math.sqrt(cx * cx + cy * cy) + 0.05;
						const normDirX = cx / dist;
						const normDirY = cy / dist;
						const explodeDist = 3.8 + Math.random() * 4.5;

						const explodedPos = new THREE.Vector3(
							cx + normDirX * explodeDist + (Math.random() - 0.5) * 2.2,
							cy + normDirY * explodeDist + (Math.random() - 0.5) * 2.2,
							(Math.random() - 0.38) * 7.5
						);

						const explodedRot = new THREE.Euler(
							(Math.random() - 0.5) * Math.PI * 3.5,
							(Math.random() - 0.5) * Math.PI * 3.5,
							(Math.random() - 0.5) * Math.PI * 3.5
						);

						shards.push({
							mesh,
							originPos: new THREE.Vector3(cx, cy, 0),
							explodedPos,
							originRot: new THREE.Euler(0, 0, 0),
							explodedRot,
							dist
						});

						logoGroup.add(mesh);
					}
				}
			}
		}

		// Particles
		const particleCount = 240;
		const particlePositions = new Float32Array(particleCount * 3);
		const particleColors = new Float32Array(particleCount * 3);
		const particleVels: { x: number; y: number; z: number }[] = [];

		for (let i = 0; i < particleCount; i++) {
			const theta = Math.random() * Math.PI * 2;
			const radius = 1.0 + Math.random() * 4.5;
			particlePositions[i * 3] = Math.cos(theta) * radius;
			particlePositions[i * 3 + 1] = Math.sin(theta) * radius;
			particlePositions[i * 3 + 2] = (Math.random() - 0.5) * 4;

			particleVels.push({
				x: (Math.random() - 0.5) * 0.018,
				y: (Math.random() - 0.5) * 0.018,
				z: (Math.random() - 0.5) * 0.018
			});

			const isGold = Math.random() > 0.28;
			if (isGold) {
				particleColors[i * 3] = 0.96;
				particleColors[i * 3 + 1] = 0.80;
				particleColors[i * 3 + 2] = 0.35;
			} else {
				particleColors[i * 3] = 0.25;
				particleColors[i * 3 + 1] = 0.85;
				particleColors[i * 3 + 2] = 1.0;
			}
		}

		const particleGeom = new THREE.BufferGeometry();
		particleGeom.setAttribute("position", new THREE.BufferAttribute(particlePositions, 3));
		particleGeom.setAttribute("color", new THREE.BufferAttribute(particleColors, 3));

		const particleMat = new THREE.PointsMaterial({
			size: 0.08,
			vertexColors: true,
			transparent: true,
			opacity: 0.7,
			blending: THREE.AdditiveBlending
		});

		const particleSystem = new THREE.Points(particleGeom, particleMat);
		scene.add(particleSystem);

		let mouseX = 0;
		let mouseY = 0;
		const handleMouseMove = (e: MouseEvent) => {
			mouseX = (e.clientX / window.innerWidth) * 2 - 1;
			mouseY = -(e.clientY / window.innerHeight) * 2 + 1;
		};
		window.addEventListener("mousemove", handleMouseMove);

		const handleResize = () => {
			const w = window.innerWidth;
			const h = window.innerHeight;
			camera.aspect = w / h;
			camera.updateProjectionMatrix();
			renderer.setSize(w, h);
		};
		window.addEventListener("resize", handleResize);

		let isExplodingParticles = false;
		let isImplodingParticles = false;

		const clock = new THREE.Clock();
		const yAxis = new THREE.Vector3(0, 1, 0);
		const dirVec = new THREE.Vector3();

		function updateBeams(elapsed: number) {
			if (beamGlobalOpacity <= 0.01 || shards.length === 0) {
				beamMeshes.forEach((b) => { b.visible = false; });
				return;
			}

			const centerPoint = new THREE.Vector3(0, 0, 0);
			for (let i = 0; i < BEAM_COUNT; i++) {
				const bMesh = beamMeshes[i];
				const s = shards[i % shards.length];
				if (!s || !s.mesh.visible) {
					bMesh.visible = false;
					continue;
				}

				bMesh.visible = true;
				const pA = (i % 2 === 0) ? centerPoint : shards[(i * 3) % shards.length].mesh.position;
				const pB = s.mesh.position;

				dirVec.subVectors(pB, pA);
				const len = dirVec.length();

				if (len < 0.05) {
					bMesh.visible = false;
					continue;
				}

				dirVec.normalize();
				bMesh.position.copy(pA);
				bMesh.quaternion.setFromUnitVectors(yAxis, dirVec);

				const radiusScale = 0.8 + 0.4 * Math.sin(elapsed * 14 + i);
				bMesh.scale.set(radiusScale, len, radiusScale);

				const mat = bMesh.material as THREE.MeshStandardMaterial;
				mat.opacity = beamGlobalOpacity * (0.65 + 0.35 * Math.sin(elapsed * 8 + i));
				mat.emissiveIntensity = 3.0 + Math.sin(elapsed * 16 + i) * 1.5;
			}
		}

		function animate() {
			reqId = requestAnimationFrame(animate);
			const elapsed = clock.getElapsedTime();

			camera.position.x += (mouseX * 0.4 - camera.position.x) * 0.05;
			camera.position.y += (mouseY * 0.4 - camera.position.y) * 0.05;
			camera.lookAt(0, 0, 0);

			const posAttr = particleGeom.attributes.position as THREE.BufferAttribute;
			const arr = posAttr.array as Float32Array;

			for (let i = 0; i < particleCount; i++) {
				const idx = i * 3;
				if (isExplodingParticles) {
					arr[idx] += particleVels[i].x * 8;
					arr[idx + 1] += particleVels[i].y * 8;
					arr[idx + 2] += particleVels[i].z * 8;
				} else if (isImplodingParticles) {
					arr[idx] += (0 - arr[idx]) * 0.08;
					arr[idx + 1] += (0 - arr[idx + 1]) * 0.08;
					arr[idx + 2] += (0 - arr[idx + 2]) * 0.08;
				} else {
					arr[idx] += Math.sin(elapsed + i) * 0.003;
					arr[idx + 1] += Math.cos(elapsed * 0.8 + i) * 0.003;
				}
			}
			posAttr.needsUpdate = true;

			updateBeams(elapsed);

			renderer.render(scene, camera);
		}
		animate();

		function startCutsceneTimeline() {
			if (tl) return;
			tl = gsap.timeline({
				onComplete: () => {
					gsap.to(container, {
						opacity: 0,
						duration: 0.7,
						ease: "power2.inOut",
						onComplete: () => {
							cleanup();
							onComplete();
						}
					});
				}
			});

			tl.set(flashOverlay, { opacity: 0 })
			  .set(letterboxTop, { height: "10vh" })
			  .set(letterboxBottom, { height: "10vh" })
			  .set([titleText, subtitleText, techBadges], { opacity: 0, y: 35 })
			  .set([shockwave1, shockwave2], { scale: 0, opacity: 0 })
			  .set(chromeGleam, { x: "-150%", opacity: 0 })
			  .set(logoGroup.scale, { x: 0.92, y: 0.92, z: 0.92 });

			// Phase 1: Hover (0.0s - 0.8s)
			tl.to(logoGroup.scale, { x: 1.0, y: 1.0, z: 1.0, duration: 0.8, ease: "power1.out" }, 0);

			// Phase 2: Stress & Charge-Up (0.8s - 1.25s)
			tl.call(() => playChargeSound(), undefined, 0.85);
			tl.to(corePointLight, { intensity: 10, duration: 0.4, ease: "power2.in" }, 0.85);
			tl.to(logoGroup.position, { x: "+=0.08", yoyo: true, repeat: 10, duration: 0.035 }, 0.85);

			// Phase 3: FLUID ULTRA-SMOOTH SHATTER (1.25s - 2.5s)
			tl.call(() => {
				playShatterSound();
				isExplodingParticles = true;
				if (intactMesh) intactMesh.visible = false;

				shards.forEach((s) => {
					s.mesh.visible = true;
					s.mesh.position.copy(s.originPos);
					s.mesh.rotation.set(0, 0, 0);

					const stagger = (s.dist / 3.5) * 0.12;

					gsap.to(s.mesh.position, {
						y: s.explodedPos.y - 4.5,
						duration: 1.45,
						delay: stagger,
						ease: "power2.in"
					});

					gsap.to(s.mesh.position, {
						x: s.explodedPos.x,
						z: s.explodedPos.z,
						duration: 1.25,
						delay: stagger,
						ease: "power3.out"
					});

					gsap.to(s.mesh.rotation, {
						x: s.explodedRot.x * 1.5,
						y: s.explodedRot.y * 1.5,
						z: s.explodedRot.z * 1.5,
						duration: 1.45,
						delay: stagger,
						ease: "power2.out"
					});
				});
			}, undefined, 1.25);

			tl.to(flashOverlay, { opacity: 0.9, duration: 0.08, ease: "power4.out" }, 1.25)
			  .to(flashOverlay, { opacity: 0, duration: 0.45, ease: "power2.out" }, 1.33)
			  .to(shockwave1, { scale: 5.5, opacity: 0.85, duration: 0.75, ease: "power2.out" }, 1.25)
			  .to(shockwave1, { opacity: 0, duration: 0.35 }, 1.65);

			// Phase 4: Zero-G Cosmic Drift & Dynamic 3D Plasma Beams (2.5s - 3.1s)
			tl.call(() => {
				isExplodingParticles = false;
			}, undefined, 2.5);

			tl.to({ val: 0 }, {
				val: 0.85,
				duration: 0.4,
				onUpdate: function() { beamGlobalOpacity = this.targets()[0].val; }
			}, 2.5);

			// Phase 5: FLUID MAGNETIC REASSEMBLY (3.1s - 4.2s)
			tl.call(() => {
				playVortexSound();
				isImplodingParticles = true;

				shards.forEach((s) => {
					const stagger = ((3.5 - s.dist) / 3.5) * 0.12;

					gsap.to(s.mesh.position, {
						x: s.originPos.x,
						y: s.originPos.y,
						z: s.originPos.z,
						duration: 1.15,
						delay: stagger,
						ease: "expo.inOut"
					});

					gsap.to(s.mesh.rotation, {
						x: 0,
						y: 0,
						z: 0,
						duration: 1.1,
						delay: stagger,
						ease: "expo.inOut"
					});
				});
			}, undefined, 3.1);

			tl.to(shockwave2, { scale: 0.1, opacity: 0.9, duration: 1.1, ease: "power3.in" }, 3.1);

			tl.to({ val: 0.85 }, {
				val: 0,
				duration: 0.45,
				onUpdate: function() { beamGlobalOpacity = this.targets()[0].val; }
			}, 3.8);

			// Phase 6: LOCK, CHROME GLEAM & TITLE REVEAL (4.2s - 4.9s)
			tl.call(() => {
				playLockSound();
				isImplodingParticles = false;
				if (intactMesh) intactMesh.visible = true;
				shards.forEach((s) => { s.mesh.visible = false; });
			}, undefined, 4.2);

			tl.to(flashOverlay, { opacity: 0.75, duration: 0.07 }, 4.2)
			  .to(flashOverlay, { opacity: 0, duration: 0.35 }, 4.27)
			  .to(logoGroup.scale, { x: 1.14, y: 1.14, z: 1.14, duration: 0.1, ease: "power4.out" }, 4.2)
			  .to(logoGroup.scale, { x: 1.0, y: 1.0, z: 1.0, duration: 0.4, ease: "elastic.out(1, 0.4)" }, 4.3)
			  .to(corePointLight, { intensity: 0, duration: 0.4 }, 4.25);

			tl.fromTo(chromeGleam, { x: "-120%", opacity: 0.85 }, { x: "220%", opacity: 0, duration: 0.75, ease: "power2.inOut" }, 4.22);

			tl.to(titleText, { opacity: 1, y: 0, duration: 0.6, ease: "back.out(2)" }, 4.25)
			  .to(subtitleText, { opacity: 1, y: 0, duration: 0.5, ease: "power2.out" }, 4.4)
			  .to(techBadges, { opacity: 1, y: 0, duration: 0.5, stagger: 0.1, ease: "power2.out" }, 4.5);

			// Phase 7: Hold & Elegant Transition (4.9s - 5.6s)
			tl.to({}, { duration: 0.85 }, 4.7);
			tl.to([letterboxTop, letterboxBottom], { height: 0, duration: 0.5, ease: "power2.inOut" }, 5.05)
			  .to([titleText, subtitleText, techBadges], { opacity: 0, y: -15, duration: 0.4, ease: "power2.in" }, 5.05)
			  .to(logoGroup.scale, { x: 1.04, y: 1.04, z: 1.04, duration: 0.5, ease: "power1.out" }, 5.05)
			  .to(container, { opacity: 0, duration: 0.6, ease: "power2.inOut" }, 5.15);
		}

		function cleanup() {
			if (reqId) cancelAnimationFrame(reqId);
			if (tl) tl.kill();
			window.removeEventListener("mousemove", handleMouseMove);
			window.removeEventListener("resize", handleResize);

			if (intactMesh) {
				intactMesh.geometry.dispose();
				if (Array.isArray(intactMesh.material)) {
					intactMesh.material.forEach((m) => m.dispose());
				} else {
					intactMesh.material.dispose();
				}
			}
			shards.forEach((s) => {
				s.mesh.geometry.dispose();
				if (Array.isArray(s.mesh.material)) {
					s.mesh.material.forEach((m) => m.dispose());
				}
			});
			particleGeom.dispose();
			particleMat.dispose();

			beamGeom.dispose();
			beamMeshes.forEach(b => {
				(b.material as THREE.Material).dispose();
			});

			if (renderer.domElement && renderer.domElement.parentNode) {
				renderer.domElement.parentNode.removeChild(renderer.domElement);
			}
			renderer.dispose();

			if (audioCtx && audioCtx.state !== "closed") {
				try { void audioCtx.close(); } catch {}
			}
		}

		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Escape" || e.key === " ") {
				cleanup();
				onComplete();
			}
		};
		window.addEventListener("keydown", handleKeyDown);

		return () => {
			window.removeEventListener("keydown", handleKeyDown);
			cleanup();
		};
	});

	function skip() {
		if (tl) tl.kill();
		onComplete();
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	bind:this={container} 
	class="fixed inset-0 z-[9999] bg-[#050608] flex flex-col items-center justify-center overflow-hidden select-none cursor-pointer"
	onclick={skip}
>
	<!-- Three.js Canvas Container -->
	<div bind:this={canvasContainer} class="absolute inset-0 z-10 pointer-events-none"></div>

	<!-- Screen Flash Overlay -->
	<div bind:this={flashOverlay} class="absolute inset-0 bg-white z-25 pointer-events-none opacity-0"></div>

	<!-- Expanding Shockwave Rings -->
	<div bind:this={shockwave1} class="absolute w-[220px] h-[220px] rounded-full border-4 border-brand-500 blur-[2px] pointer-events-none z-15 shadow-[0_0_80px_rgba(226,184,107,0.8)]"></div>
	<div bind:this={shockwave2} class="absolute w-[500px] h-[500px] rounded-full border-2 border-cyan-400 blur-[1px] pointer-events-none z-15 shadow-[0_0_60px_rgba(56,189,248,0.6)]"></div>

	<!-- Chrome Gleam Sweep -->
	<div class="absolute inset-0 z-20 pointer-events-none overflow-hidden flex items-center justify-center">
		<div bind:this={chromeGleam} class="w-[600px] h-[200px] bg-gradient-to-r from-transparent via-white/80 to-transparent skew-x-[-35deg] blur-md"></div>
	</div>

	<!-- Cinematic Letterbox Bars -->
	<div bind:this={letterboxTop} class="absolute top-0 left-0 right-0 bg-black z-30 pointer-events-none"></div>
	<div bind:this={letterboxBottom} class="absolute bottom-0 left-0 right-0 bg-black z-30 pointer-events-none"></div>

	<!-- Cyber HUD Background Elements -->
	<div class="absolute inset-0 pointer-events-none bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-amber-500/10 via-transparent to-black z-0">
		<div class="absolute inset-0 bg-[radial-gradient(#e2b86b_1px,transparent_1px)] [background-size:32px_32px] opacity-15"></div>
	</div>

	<!-- Modern Typography Overlay -->
	<div class="absolute bottom-20 z-20 text-center space-y-2 pointer-events-none">
		<h1 bind:this={titleText} class="text-5xl md:text-7xl font-black text-transparent bg-clip-text bg-gradient-to-b from-white via-neutral-100 to-brand-500 tracking-[0.28em] uppercase drop-shadow-[0_0_40px_rgba(226,184,107,0.65)] font-sans">
			LUXMC
		</h1>
		<p bind:this={subtitleText} class="text-xs md:text-sm font-mono tracking-[0.45em] text-brand-500 uppercase font-bold drop-shadow-[0_0_15px_rgba(226,184,107,0.5)]">
			MINECRAFT LAUNCHER • NEXT GEN V0.2.0
		</p>

		<!-- Badges -->
		<div bind:this={techBadges} class="flex items-center justify-center gap-3 pt-2">
			<span class="px-3 py-1 rounded-full text-[10px] font-mono tracking-widest uppercase bg-amber-500/10 border border-amber-500/30 text-amber-300 backdrop-blur-md">
				VULKAN READY
			</span>
			<span class="px-3 py-1 rounded-full text-[10px] font-mono tracking-widest uppercase bg-cyan-500/10 border border-cyan-500/30 text-cyan-300 backdrop-blur-md">
				TAURI 2 & RUST
			</span>
			<span class="px-3 py-1 rounded-full text-[10px] font-mono tracking-widest uppercase bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 backdrop-blur-md">
				DISCORD RPC
			</span>
		</div>
	</div>

	<!-- Mute/Unmute Audio Button -->
	<button 
		onclick={(e) => { e.stopPropagation(); isMuted = !isMuted; }}
		class="absolute top-6 right-6 z-40 p-2.5 rounded-full bg-black/40 border border-amber-500/30 text-amber-400 hover:bg-black/60 transition-all cursor-pointer"
		title={isMuted ? "Ativar Áudio" : "Mutar Áudio"}
	>
		{#if isMuted}
			<VolumeX class="w-5 h-5" />
		{:else}
			<Volume2 class="w-5 h-5" />
		{/if}
	</button>
</div>
