<script lang="ts">
	import { untrack } from "svelte";
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { 
		Disc, 
		Music, 
		Volume2, 
		Play, 
		Pause, 
		RotateCcw,
		FolderOpen,
		Sparkles
	} from "lucide-svelte";
	import Modal from "$lib/components/ui/Modal.svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import AudioVisualizer from "$lib/components/ui/AudioVisualizer.svelte";

	type Track = {
		id: string;
		title: string;
		author: string;
		discColor: string;
		url: string;
	};

	type Props = {
		open: boolean;
		onClose: () => void;
	};

	let { open, onClose }: Props = $props();

	const playlist: Track[] = $derived([
		{
			id: "pigstep",
			title: "Pigstep",
			author: "Lena Raine",
			discColor: "rgb(var(--warning))",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/pigstep.ogg"
		},
		{
			id: "otherside",
			title: "Otherside",
			author: "Lena Raine",
			discColor: "rgb(var(--info))",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/otherside.ogg"
		},
		{
			id: "cat",
			title: "Cat",
			author: "C418",
			discColor: "rgb(var(--success))",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/cat.ogg"
		},
		{
			id: "chirp",
			title: "Chirp",
			author: "C418",
			discColor: "rgb(var(--danger))",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/chirp.ogg"
		},
		{
			id: "relic",
			title: "Relic",
			author: "Aaron Cherof",
			discColor: "rgb(var(--loader-quilt))",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/relic.ogg"
		}
	]);

	let activeTrack = $state<Track>(untrack(() => playlist[0]));
</script>

<Modal isOpen={open} {onClose} title={uiText("ui.28786aa5dd0af384")}>
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-fg/70 leading-relaxed">
			{uiText("ui.79182cc08de11373")}
		</p>

		<!-- Visualizer Card -->
		<AudioVisualizer url={activeTrack.url} title={`${activeTrack.title} — ${activeTrack.author}`} />

		<!-- Playlist Grid -->
		<div class="flex flex-col gap-2">
			<span class="text-[11px] font-bold text-fg/50 uppercase tracking-wider">{uiText("ui.1f4e6beded2357c1")}</span>
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
				{#each playlist as track}
					<button
						type="button"
						class="flex items-center gap-3 p-2.5 rounded-xl border text-left transition-[color,background-color,border-color,box-shadow,transform,opacity] cursor-pointer {activeTrack.id === track.id ? 'bg-brand-400/15 border-brand-400/50' : 'bg-bg-subtle border-fg/5 hover:border-fg/20'}"
						onclick={() => activeTrack = track}
					>
						<div 
							class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 shadow-md"
							style="background-color: {track.discColor}25; border: 1px solid {track.discColor}60; color: {track.discColor};"
						>
							<Disc class="w-4 h-4 {activeTrack.id === track.id ? 'animate-spin' : ''}" />
						</div>

						<div class="min-w-0 flex-1">
							<h4 class="text-fg font-bold text-xs truncate">{track.title}</h4>
							<p class="text-[10px] text-fg/50 truncate">{track.author}</p>
						</div>

						{#if activeTrack.id === track.id}
							<span class="text-[10px] font-bold text-brand-400 px-2 py-0.5 rounded-full bg-brand-400/20">
								{uiText("ui.b6f1f2a32314859e")}
							</span>
						{/if}
					</button>
				{/each}
			</div>
		</div>

		<div class="flex items-center justify-end pt-2 border-t border-fg/5">
			<Button variant="secondary" size="sm" onclick={onClose}>
				{uiText("statusBanner.dismiss")}
			</Button>
		</div>
	</div>
</Modal>
