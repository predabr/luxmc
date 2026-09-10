<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import gsap from "gsap";

	let { onComplete = () => {} }: { onComplete?: () => void } = $props();

	let container = $state<HTMLElement | null>(null);
	let canvas = $state<HTMLCanvasElement | null>(null);
	let animId: number | null = null;
	let tl: gsap.core.Timeline | null = null;
	let completed = false;

	interface Particle {
		x: number;
		y: number;
		vx: number;
		vy: number;
		radius: number;
		alpha: number;
	}

	function finish() {
		if (completed) return;
		completed = true;
		if (animId) {
			cancelAnimationFrame(animId);
			animId = null;
		}
		if (tl) {
			tl.kill();
			tl = null;
		}
		onComplete();
	}

	function playChime() {
		try {
			const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
			if (!AudioCtx) return;
			const ctx = new AudioCtx();
			const now = ctx.currentTime;

			// Low sub sweep
			const sub = ctx.createOscillator();
			const subGain = ctx.createGain();
			sub.type = "sine";
			sub.frequency.setValueAtTime(120, now);
			sub.frequency.exponentialRampToValueAtTime(320, now + 0.25);
			subGain.gain.setValueAtTime(0.01, now);
			subGain.gain.exponentialRampToValueAtTime(0.2, now + 0.08);
			subGain.gain.exponentialRampToValueAtTime(0.001, now + 0.45);
			sub.connect(subGain);
			subGain.connect(ctx.destination);
			sub.start(now);
			sub.stop(now + 0.45);

			// Clean crystal bell chime
			const bell = ctx.createOscillator();
			const bellGain = ctx.createGain();
			bell.type = "triangle";
			bell.frequency.setValueAtTime(659.25, now); // E5
			bell.frequency.setValueAtTime(987.77, now + 0.08); // B5
			bell.frequency.setValueAtTime(1318.51, now + 0.16); // E6
			bellGain.gain.setValueAtTime(0.01, now);
			bellGain.gain.exponentialRampToValueAtTime(0.25, now + 0.12);
			bellGain.gain.exponentialRampToValueAtTime(0.001, now + 0.7);
			bell.connect(bellGain);
			bellGain.connect(ctx.destination);
			bell.start(now);
			bell.stop(now + 0.7);
		} catch {}
	}

	onMount(() => {
		const ctx = canvas?.getContext("2d");
		if (!ctx || !canvas) {
			finish();
			return;
		}

		// Fail-safe timeout: never block app loading
		const fallbackTimer = setTimeout(() => {
			finish();
		}, 2600);

		// Keydown listener to skip immediately
		const keyHandler = (e: KeyboardEvent) => {
			if (e.key === " " || e.key === "Enter" || e.key === "Escape") {
				finish();
			}
		};
		window.addEventListener("keydown", keyHandler);

		const img = new Image();
		img.src = "/logo.png";
		img.onload = () => {
			if (completed) return;
			startAnimation(img, ctx, canvas!);
		};
		img.onerror = () => {
			clearTimeout(fallbackTimer);
			finish();
		};

		return () => {
			clearTimeout(fallbackTimer);
			window.removeEventListener("keydown", keyHandler);
			finish();
		};
	});

	function startAnimation(img: HTMLImageElement, ctx: CanvasRenderingContext2D, cvs: HTMLCanvasElement) {
		const dpr = Math.min(window.devicePixelRatio || 1, 2);
		const width = window.innerWidth;
		const height = window.innerHeight;

		cvs.width = width * dpr;
		cvs.height = height * dpr;
		ctx.scale(dpr, dpr);

		const logoSize = Math.min(width, height) * 0.28;
		const half = logoSize / 2;
		const centerX = width / 2;
		const centerY = height / 2;

		// 24 glowing ambient embers
		const particles: Particle[] = Array.from({ length: 24 }, () => ({
			x: Math.random() * width,
			y: Math.random() * height,
			vx: (Math.random() - 0.5) * 0.5,
			vy: (Math.random() - 0.5) * 0.5,
			radius: Math.random() * 2 + 1,
			alpha: Math.random() * 0.5 + 0.2
		}));

		const quadrants = [
			{ sx: 0, sy: 0, dx: -half, dy: -half, x: -half, y: -half, rot: 0, tx: -half - 110, ty: -half - 110, troat: -Math.PI },
			{ sx: img.width / 2, sy: 0, dx: 0, dy: -half, x: 0, y: -half, rot: 0, tx: 110, ty: -half - 110, troat: Math.PI },
			{ sx: 0, sy: img.height / 2, dx: -half, dy: 0, x: -half, y: 0, rot: 0, tx: -half - 110, ty: 110, troat: Math.PI },
			{ sx: img.width / 2, sy: img.height / 2, dx: 0, dy: 0, x: 0, y: 0, rot: 0, tx: 110, ty: 110, troat: -Math.PI }
		];

		let auraGlow = 0;
		let flashAlpha = 0;
		let globalScale = 1;
		let globalOpacity = 1;

		function render() {
			ctx.clearRect(0, 0, width, height);

			// Radial ambient background glow (fast, no heavy loops)
			const grad = ctx.createRadialGradient(centerX, centerY, 5, centerX, centerY, logoSize * 1.5);
			grad.addColorStop(0, `rgba(216, 188, 152, ${0.12 + auraGlow * 0.15})`);
			grad.addColorStop(0.6, `rgba(216, 188, 152, 0.02)`);
			grad.addColorStop(1, "transparent");
			ctx.fillStyle = grad;
			ctx.fillRect(0, 0, width, height);

			// Render ambient particles
			ctx.fillStyle = "rgba(216, 188, 152, 0.6)";
			for (let i = 0; i < particles.length; i++) {
				const p = particles[i];
				p.x += p.vx;
				p.y += p.vy;
				if (p.x < 0) p.x = width;
				if (p.x > width) p.x = 0;
				if (p.y < 0) p.y = height;
				if (p.y > height) p.y = 0;

				ctx.beginPath();
				ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
				ctx.fill();
			}

			// Render 4 Quadrants
			ctx.save();
			ctx.translate(centerX, centerY);
			ctx.scale(globalScale, globalScale);
			ctx.globalAlpha = globalOpacity;

			for (let i = 0; i < quadrants.length; i++) {
				const q = quadrants[i];
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
			}

			// Central golden flash on snap
			if (flashAlpha > 0) {
				const flashGrad = ctx.createRadialGradient(0, 0, 0, 0, 0, logoSize);
				flashGrad.addColorStop(0, `rgba(255, 240, 210, ${flashAlpha * 0.8})`);
				flashGrad.addColorStop(0.5, `rgba(216, 188, 152, ${flashAlpha * 0.3})`);
				flashGrad.addColorStop(1, "transparent");
				ctx.fillStyle = flashGrad;
				ctx.beginPath();
				ctx.arc(0, 0, logoSize, 0, Math.PI * 2);
				ctx.fill();
			}

			ctx.restore();
		}

		tl = gsap.timeline({
			onUpdate: render,
			onComplete: () => {
				if (container) {
					gsap.to(container, {
						opacity: 0,
						duration: 0.25,
						ease: "power2.out",
						onComplete: finish
					});
				} else {
					finish();
				}
			}
		});

		// 1. Initial burst outwards (0s -> 0.9s)
		quadrants.forEach((q) => {
			tl!.to(q, {
				x: q.tx,
				y: q.ty,
				rot: q.troat,
				duration: 0.9,
				ease: "power2.out"
			}, 0.1);
		});

		// 2. Smooth continuous return & snap (0.9s -> 1.7s) - NO FREEZING!
		quadrants.forEach((q) => {
			tl!.to(q, {
				x: q.dx,
				y: q.dy,
				rot: 0,
				duration: 0.8,
				ease: "back.out(1.4)"
			}, 0.9);
		});

		// Chime & Flash on snap impact (1.68s)
		tl.call(() => {
			playChime();
		}, [], 1.68);

		tl.to({ f: 1, a: 0 }, {
			f: 0,
			a: 1,
			duration: 0.4,
			ease: "power2.out",
			onUpdate: function() {
				flashAlpha = this.targets()[0].f;
				auraGlow = this.targets()[0].a;
			}
		}, 1.68);

		// 3. Final smooth expansion into main app (1.9s -> 2.3s)
		tl.to({ s: 1, op: 1 }, {
			s: 1.1,
			op: 0,
			duration: 0.38,
			ease: "power2.inOut",
			onUpdate: function() {
				globalScale = this.targets()[0].s;
				globalOpacity = this.targets()[0].op;
			}
		}, 1.95);
	}

	onDestroy(() => {
		finish();
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	bind:this={container} 
	class="fixed inset-0 z-[9999] bg-[#090a0d] flex flex-col items-center justify-center overflow-hidden select-none cursor-pointer"
	onclick={finish}
>
	<canvas bind:this={canvas} class="absolute inset-0 w-full h-full pointer-events-none"></canvas>

	<!-- Sleek Brand Typography -->
	<div class="absolute bottom-12 z-20 text-center space-y-1 pointer-events-none">
		<h1 class="text-2xl font-black text-white tracking-[0.35em] uppercase drop-shadow-[0_0_20px_rgba(216,188,152,0.4)]">
			LUXMC
		</h1>
		<p class="text-[11px] font-mono tracking-[0.3em] text-[#d8bc98] uppercase font-bold">
			v1.0.0-BETA · LINUX LAUNCHER
		</p>
	</div>

	<!-- Skip Prompt (bottom right) -->
	<div class="absolute bottom-6 right-6 z-20 pointer-events-none">
		<span class="text-[10px] text-white/30 font-medium tracking-wider bg-white/5 border border-white/10 px-3 py-1.5 rounded-full">
			Pressione Espaço ou Clique para Pular
		</span>
	</div>
</div>
