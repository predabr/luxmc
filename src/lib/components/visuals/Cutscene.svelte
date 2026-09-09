<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { fade } from "svelte/transition";
	import gsap from "gsap";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let container: HTMLElement;
	let canvas: HTMLCanvasElement;
	let animId: number;
	let tl: gsap.core.Timeline;

	// Background particles structure
	interface Particle {
		x: number;
		y: number;
		vx: number;
		vy: number;
		radius: number;
		alpha: number;
		pulseSpeed: number;
	}

	onMount(() => {
		const ctx = canvas?.getContext("2d");
		if (!ctx) {
			onComplete();
			return;
		}

		// Safeguard: Ensure splash/cutscene never blocks app startup indefinitely
		const fallbackTimer = setTimeout(() => {
			onComplete();
		}, 4200);

		const img = new Image();
		img.src = "/logo.png";
		img.onload = () => {
			runCutscene(img, ctx);
		};
		img.onerror = () => {
			clearTimeout(fallbackTimer);
			onComplete();
		};

		return () => {
			clearTimeout(fallbackTimer);
			if (animId) cancelAnimationFrame(animId);
			if (tl) tl.kill();
		};
	});

	function playCutsceneAudio() {
		try {
			const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
			if (!AudioCtx) return;
			const ctx = new AudioCtx();
			const now = ctx.currentTime;

			// Low sub sweep / lock sound
			const subOsc = ctx.createOscillator();
			const subGain = ctx.createGain();
			subOsc.type = "sine";
			subOsc.frequency.setValueAtTime(110, now);
			subOsc.frequency.exponentialRampToValueAtTime(350, now + 0.35);
			subGain.gain.setValueAtTime(0.01, now);
			subGain.gain.exponentialRampToValueAtTime(0.3, now + 0.1);
			subGain.gain.exponentialRampToValueAtTime(0.001, now + 0.6);
			subOsc.connect(subGain);
			subGain.connect(ctx.destination);
			subOsc.start(now);
			subOsc.stop(now + 0.6);

			// High golden chime harmonic
			const chimeOsc = ctx.createOscillator();
			const chimeGain = ctx.createGain();
			chimeOsc.type = "triangle";
			chimeOsc.frequency.setValueAtTime(523.25, now + 0.1); // C5
			chimeOsc.frequency.setValueAtTime(659.25, now + 0.2); // E5
			chimeOsc.frequency.setValueAtTime(1046.5, now + 0.3); // C6
			chimeGain.gain.setValueAtTime(0.01, now + 0.1);
			chimeGain.gain.exponentialRampToValueAtTime(0.22, now + 0.25);
			chimeGain.gain.exponentialRampToValueAtTime(0.001, now + 0.85);
			chimeOsc.connect(chimeGain);
			chimeGain.connect(ctx.destination);
			chimeOsc.start(now + 0.1);
			chimeOsc.stop(now + 0.85);
		} catch (e) {
			// Audio context restricted or muted
		}
	}

	function runCutscene(img: HTMLImageElement, ctx: CanvasRenderingContext2D) {
		const dpr = window.devicePixelRatio || 1;
		const width = window.innerWidth;
		const height = window.innerHeight;

		canvas.width = width * dpr;
		canvas.height = height * dpr;
		ctx.scale(dpr, dpr);

		const logoSize = Math.min(width, height) * 0.32;
		const half = logoSize / 2;
		const centerX = width / 2;
		const centerY = height / 2;

		// Generate ambient golden particles
		const particles: Particle[] = Array.from({ length: 45 }, () => ({
			x: Math.random() * width,
			y: Math.random() * height,
			vx: (Math.random() - 0.5) * 0.4,
			vy: (Math.random() - 0.5) * 0.4,
			radius: Math.random() * 2.2 + 1.0,
			alpha: Math.random() * 0.5 + 0.2,
			pulseSpeed: Math.random() * 0.02 + 0.01
		}));

		// 4 Quadrants: TL (top-left), TR (top-right), BL (bottom-left), BR (bottom-right)
		const quadrants = [
			{ sx: 0, sy: 0, dx: -half, dy: -half, x: -half, y: -half, rot: 0, targetX: -half - 140, targetY: -half - 140, targetRot: -Math.PI * 2 },
			{ sx: img.width / 2, sy: 0, dx: 0, dy: -half, x: 0, y: -half, rot: 0, targetX: 140, targetY: -half - 140, targetRot: Math.PI * 2 },
			{ sx: 0, sy: img.height / 2, dx: -half, dy: 0, x: -half, y: 0, rot: 0, targetX: -half - 140, targetY: 140, targetRot: Math.PI * 2 },
			{ sx: img.width / 2, sy: img.height / 2, dx: 0, dy: 0, x: 0, y: 0, rot: 0, targetX: 140, targetY: 140, targetRot: -Math.PI * 2 }
		];

		let globalScale = 1.0;
		let globalAlpha = 1.0;
		let auraPulse = 0;

		function render() {
			ctx.clearRect(0, 0, width, height);

			// 1. Draw Ambient Background Grid & Radial Pulse
			auraPulse += 0.02;
			const pulseRadius = (Math.min(width, height) * 0.35) + Math.sin(auraPulse) * 12;
			const grad = ctx.createRadialGradient(centerX, centerY, 10, centerX, centerY, pulseRadius);
			grad.addColorStop(0, "rgba(226, 184, 107, 0.12)");
			grad.addColorStop(0.5, "rgba(226, 184, 107, 0.03)");
			grad.addColorStop(1, "rgba(7, 8, 10, 0)");

			ctx.fillStyle = grad;
			ctx.fillRect(0, 0, width, height);

			// Subtle Grid Lines
			ctx.strokeStyle = "rgba(255, 255, 255, 0.02)";
			ctx.lineWidth = 1;
			const gridSize = 40;
			for (let x = 0; x < width; x += gridSize) {
				ctx.beginPath();
				ctx.moveTo(x, 0);
				ctx.lineTo(x, height);
				ctx.stroke();
			}
			for (let y = 0; y < height; y += gridSize) {
				ctx.beginPath();
				ctx.moveTo(0, y);
				ctx.lineTo(width, y);
				ctx.stroke();
			}

			// 2. Draw Floating Particles
			particles.forEach((p) => {
				p.x += p.vx;
				p.y += p.vy;
				if (p.x < 0) p.x = width;
				if (p.x > width) p.x = 0;
				if (p.y < 0) p.y = height;
				if (p.y > height) p.y = 0;

				ctx.save();
				ctx.fillStyle = `rgba(226, 184, 107, ${p.alpha * globalAlpha})`;
				ctx.beginPath();
				ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
				ctx.fill();
				ctx.restore();
			});

			// 3. Draw 4 Quadrants
			ctx.save();
			ctx.translate(centerX, centerY);
			ctx.scale(globalScale, globalScale);
			ctx.globalAlpha = globalAlpha;

			quadrants.forEach((q) => {
				ctx.save();
				const originX = q.x + half / 2;
				const originY = q.y + half / 2;

				ctx.translate(originX, originY);
				ctx.rotate(q.rot);
				ctx.translate(-originX, -originY);

				ctx.drawImage(
					img,
					q.sx, q.sy, img.width / 2, img.height / 2,
					q.x, q.y, half, half
				);
				ctx.restore();
			});

			ctx.restore();
		}

		tl = gsap.timeline({
			onUpdate: render,
			onComplete: () => {
				gsap.to(container, {
					opacity: 0,
					duration: 0.4,
					ease: "power2.inOut",
					onComplete: () => {
						onComplete();
					}
				});
			}
		});

		// Timeline Sequence:
		// Step 1 (0.0s - 0.3s): Initial Centered Hold
		render();

		// Step 2 (0.3s - 2.3s = 2.0 SECONDS DURATION):
		// 4 pieces separate AND rotate 360 degrees for exactly 2 seconds!
		quadrants.forEach((q) => {
			tl.to(q, {
				x: q.targetX,
				y: q.targetY,
				rot: q.targetRot,
				duration: 2.0,
				ease: "power2.out"
			}, 0.3);
		});

		// Step 3 (2.3s - 3.1s = 0.8 SECONDS PAUSE):
		// Pieces STOP spinning and hold position in separated state!
		tl.to({}, { duration: 0.8 });

		// Step 4 (3.1s - 4.1s = 1.0 SECOND REASSEMBLY):
		// Pieces move back into center position seamlessly without rotation to assemble perfectly!
		quadrants.forEach((q) => {
			tl.to(q, {
				x: q.dx,
				y: q.dy,
				rot: q.targetRot > 0 ? Math.PI * 2 : -Math.PI * 2,
				duration: 1.0,
				ease: "power3.inOut"
			}, 3.1);
		});

		// Play Synthesized Audio Chime exactly as reassembly completes!
		tl.call(() => {
			playCutsceneAudio();
		}, [], 4.1);

		// Step 5 (4.1s - 4.6s): Smooth scale up & fade reveal to main launcher
		tl.to({ scale: 1 }, {
			scale: 1.15,
			duration: 0.5,
			ease: "power2.out",
			onUpdate: function() {
				globalScale = this.targets()[0].scale;
			}
		}, 4.1);

		tl.to({ alpha: 1 }, {
			alpha: 0,
			duration: 0.4,
			ease: "power2.inOut",
			onUpdate: function() {
				globalAlpha = this.targets()[0].alpha;
			}
		}, 4.2);
	}

	function skip() {
		if (tl) tl.kill();
		onComplete();
	}

	onDestroy(() => {
		if (animId) cancelAnimationFrame(animId);
		if (tl) tl.kill();
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	bind:this={container} 
	class="fixed inset-0 z-[9999] bg-[#07080a] flex flex-col items-center justify-center overflow-hidden select-none cursor-pointer"
	onclick={skip}
>
	<canvas bind:this={canvas} class="absolute inset-0 w-full h-full pointer-events-none"></canvas>

	<!-- Sleek Title & Version Badge -->
	<div class="absolute bottom-16 z-20 text-center space-y-1.5 pointer-events-none">
		<h1 class="text-3xl font-black text-white tracking-[0.35em] uppercase drop-shadow-[0_0_20px_rgba(226,184,107,0.4)]">
			LUXMC
		</h1>
		<p class="text-xs font-mono tracking-[0.4em] text-brand-500 uppercase font-bold">
			v0.7.0-BETA · MINECRAFT LAUNCHER
		</p>
	</div>
</div>
