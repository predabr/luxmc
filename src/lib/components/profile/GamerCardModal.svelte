<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
	import { backOut, quintOut } from "svelte/easing";
	import { onMount } from "svelte";
	import { fade, scale } from "svelte/transition";
	import {
		Share2,
		Copy,
		Download,
		Check,
		Sparkles,
		X,
		Trophy,
		Clock,
		Flame,
		Gamepad2
	} from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { profiles } from "$lib/stores/profiles.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { achievements, ACHIEVEMENTS_LIST } from "$lib/stores/achievements.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { playSound } from "$lib/utils/sound";

	let { isOpen = $bindable(false), onClose }: { isOpen: boolean; onClose: () => void } = $props();

	let canvasElem: HTMLCanvasElement | null = $state(null);
	let isGenerating = $state(true);
	let cardDataUrl = $state<string | null>(null);
	let copied = $state(false);

	const username = $derived(account.value?.username || "Jogador Luxmc");
	const totalTime = $derived(gamingStats.formattedTotalTime || "0h 00m");
	const activeProfileName = $derived(profiles.active?.name || "Minecraft Vanilla");
	const unlockedCount = $derived(achievements.unlocked.length);
	const totalAchievements = $derived(ACHIEVEMENTS_LIST.length || 5);

	$effect(() => {
		if (isOpen) {
			const timer = setTimeout(renderGamerCard, 100);
            return () => clearTimeout(timer);
		}
	});

	function color(token: string, opacity = 1): string {
		const channels = getComputedStyle(document.documentElement).getPropertyValue(`--${token}`).trim();
		return `rgb(${channels} / ${opacity})`;
	}

	function renderGamerCard() {
		if (!canvasElem) return;
		isGenerating = true;

		const ctx = canvasElem.getContext("2d");
		if (!ctx) return;

		const width = 800;
		const height = 450;
		canvasElem.width = width;
		canvasElem.height = height;

		const bgGradient = ctx.createLinearGradient(0, 0, width, height);
		bgGradient.addColorStop(0, color("bg-subtle"));
		bgGradient.addColorStop(0.5, color("bg-elevated"));
		bgGradient.addColorStop(1, color("bg"));
		ctx.fillStyle = bgGradient;
		ctx.fillRect(0, 0, width, height);

		const glowA = ctx.createRadialGradient(0, 0, 10, 0, 0, 350);
		glowA.addColorStop(0, color("brand-400", 0.25));
		glowA.addColorStop(1, color("brand-400", 0));
		ctx.fillStyle = glowA;
		ctx.fillRect(0, 0, width, height);

		const glowB = ctx.createRadialGradient(width, height, 10, width, height, 380);
		glowB.addColorStop(0, color("brand-600", 0.2));
		glowB.addColorStop(1, color("brand-600", 0));
		ctx.fillStyle = glowB;
		ctx.fillRect(0, 0, width, height);

		ctx.strokeStyle = color("fg", 0.12);
		ctx.lineWidth = 2;
		ctx.strokeRect(1, 1, width - 2, height - 2);

		ctx.strokeStyle = color("brand-400", 0.25);
		ctx.lineWidth = 1;
		ctx.strokeRect(12, 12, width - 24, height - 24);

		ctx.fillStyle = color("brand-400");
		ctx.font = "900 13px system-ui, sans-serif";
		ctx.fillText("LUXMC LAUNCHER", 40, 50);

		ctx.fillStyle = color("fg", 0.4);
		ctx.font = "700 11px system-ui, sans-serif";
		ctx.fillText("OFFICIAL GAMER PASSPORT", 195, 50);

		ctx.fillStyle = color("bg-overlay", 0.4);
		ctx.fillRect(40, 80, 140, 140);
		ctx.strokeStyle = color("brand-400", 0.5);
		ctx.lineWidth = 2;
		ctx.strokeRect(40, 80, 140, 140);

		const avatarImg = new Image();
		avatarImg.crossOrigin = "anonymous";
		avatarImg.src = account.value?.skinUrl || "/grass_head.png";
		avatarImg.onload = () => {
			ctx.imageSmoothingEnabled = false;
			ctx.drawImage(avatarImg, 8, 8, 8, 8, 48, 88, 124, 124);
			finalizeCard();
		};
		avatarImg.onerror = () => {
			finalizeCard();
		};

		function finalizeCard() {
			if (!ctx) return;
			ctx.fillStyle = color("fg");
			ctx.font = "900 28px system-ui, sans-serif";
			ctx.fillText(username, 210, 120);

			ctx.fillStyle = color("fg", 0.5);
			ctx.font = "600 13px system-ui, sans-serif";
			const isOnlineAcc = account.value?.minecraftToken && !account.value?.id.startsWith("offline_");
			ctx.fillText(isOnlineAcc ? uiText("ui.eca633249b985c66") : "⚡ Jogador Luxmc", 210, 145);

			ctx.fillStyle = color("fg", 0.1);
			ctx.fillRect(210, 165, 540, 4);
			ctx.fillStyle = color("brand-400");
			ctx.fillRect(210, 165, 340, 4);

			drawStatBox(ctx, 40, 250, 220, 120, uiText("ui.7bae9616f3a82389"), totalTime, uiText("ui.b5842e496b99eaf6"), color("brand-400"));
			drawStatBox(ctx, 290, 250, 220, 120, uiText("ui.92adde8a277eb570"), activeProfileName, uiText("ui.5e549533fc47bd5d"), color("brand-300"));
			drawStatBox(ctx, 540, 250, 220, 120, "CONQUISTAS", `${unlockedCount} / ${totalAchievements}`, "Desafios desbloqueados", color("success"));

			ctx.fillStyle = color("fg", 0.3);
			ctx.font = "500 11px system-ui, sans-serif";
			ctx.fillText("Gerado pelo Luxmc Launcher · Linux-First Gaming · github.com/predabr/luxmc", 40, 415);

			if (canvasElem) {
				cardDataUrl = canvasElem.toDataURL("image/png");
			}
			isGenerating = false;
		}
	}

	function drawStatBox(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, label: string, value: string, sub: string, accent: string) {
		ctx.fillStyle = color("bg-subtle", 0.85);
		ctx.fillRect(x, y, w, h);
		ctx.strokeStyle = color("fg", 0.08);
		ctx.lineWidth = 1;
		ctx.strokeRect(x, y, w, h);

		ctx.fillStyle = accent;
		ctx.font = "800 10px system-ui, sans-serif";
		ctx.fillText(label, x + 16, y + 28);

		ctx.fillStyle = color("fg");
		ctx.font = "900 18px system-ui, sans-serif";
		const truncVal = value.length > 18 ? value.slice(0, 17) + "..." : value;
		ctx.fillText(truncVal, x + 16, y + 62);

		ctx.fillStyle = color("fg", 0.4);
		ctx.font = "500 10px system-ui, sans-serif";
		ctx.fillText(sub, x + 16, y + 92);
	}

	async function handleCopyCard() {
        if (!canvasElem) return;
        try {
            const blob = await new Promise<Blob>((resolve, reject) => {
                canvasElem!.toBlob(value => value ? resolve(value) : reject(new Error(uiText("ui.9c6aa26cc4db2931"))), "image/png");
            });
            await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
            copied = true;
            playSound("click");
            toast(uiText("ui.3c6f6a5224cdab05"), "success");
            setTimeout(() => copied = false, 2500);
        } catch {
            toast(uiText("ui.6f18281b59f3d6ce"), "warning");
        }
    }

	function handleDownloadCard() {
		if (!cardDataUrl) return;
		const a = document.createElement("a");
		a.href = cardDataUrl;
		a.download = `luxmc_card_${username.toLowerCase()}.png`;
		a.click();
		playSound("click");
		toast(uiText("ui.89b744ea9ffeecaf"), "success");
	}
</script>

{#if isOpen}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-bg-overlay/85 backdrop-blur-md select-none" in:fade={{ easing: quintOut, duration: 220 }}>
		<div class="w-full max-w-4xl rounded-3xl bg-bg-elevated border border-fg/15 p-7 shadow-2xl space-y-6 relative overflow-hidden" in:scale={{ easing: backOut, start: 0.95, duration: 260 }}>
			<div class="absolute -top-20 -left-20 w-64 h-64 bg-brand-400/15 rounded-full blur-3xl pointer-events-none"></div>
			<div class="absolute -bottom-20 -right-20 w-64 h-64 bg-brand-500/15 rounded-full blur-3xl pointer-events-none"></div>
			<div class="flex items-center justify-between border-b border-fg/10 pb-4 relative z-10">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-2xl bg-brand-400/15 border border-brand-400/30 flex items-center justify-center text-brand-400">
						<Sparkles class="w-5 h-5" />
					</div>
					<div>
						<h2 class="text-base font-black text-fg tracking-tight">{uiText("ui.6f8d215ede0cf5b5")}</h2>
						<p class="text-xs text-fg/50">{uiText("ui.ef2fb2b53a3c44c2")}</p>
					</div>
				</div>

				<button
					type="button"
					class={launcherButton({ variant: "secondary", size: "icon", class: "" })}
					onclick={onClose}
					aria-label={uiText("ui.236becab291a25ea")}
				>
					<X class="w-5 h-5" />
				</button>
			</div>
			<div class="flex flex-col items-center justify-center relative z-10">
				<div class="w-full max-w-[800px] overflow-hidden rounded-2xl shadow-2xl border border-fg/15 bg-bg-overlay/50">
					<canvas bind:this={canvasElem} class="w-full h-auto block"></canvas>
				</div>
			</div>
			<div class="flex flex-col sm:flex-row items-center justify-between gap-3 pt-2 border-t border-fg/10 relative z-10">
				<span class="text-xs text-fg/40 font-mono">{uiText("ui.e88fee0792748319")}</span>

				<div class="flex items-center gap-3 w-full sm:w-auto justify-end">
					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
						onclick={handleDownloadCard}
						disabled={isGenerating}
					>
						<Download class="w-4 h-4 text-brand-400" />
						<span>{uiText("ui.285d56738d52c27d")}</span>
					</button>

					<button
						type="button"
						class={launcherButton({ variant: "secondary", size: "sm", class: "from-brand-400 via-brand-400 to-brand-400 hover:from-brand-400 hover:to-brand-400 uppercase tracking-wider flex items-center gap-2" })}
						onclick={handleCopyCard}
						disabled={isGenerating}
					>
						{#if copied}
							<Check class="w-4 h-4 stroke-[3]" />
							<span>{uiText("ui.a8fe0fc805d5fd50")}</span>
						{:else}
							<Copy class="w-4 h-4" />
							<span>{uiText("ui.639f190727af2da0")}</span>
						{/if}
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
