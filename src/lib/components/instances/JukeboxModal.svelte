<script lang="ts">
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

	const playlist: Track[] = [
		{
			id: "pigstep",
			title: "Pigstep",
			author: "Lena Raine",
			discColor: "#d97706",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/pigstep.ogg"
		},
		{
			id: "otherside",
			title: "Otherside",
			author: "Lena Raine",
			discColor: "#0284c7",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/otherside.ogg"
		},
		{
			id: "cat",
			title: "Cat",
			author: "C418",
			discColor: "#16a34a",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/cat.ogg"
		},
		{
			id: "chirp",
			title: "Chirp",
			author: "C418",
			discColor: "#dc2626",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/chirp.ogg"
		},
		{
			id: "relic",
			title: "Relic",
			author: "Aaron Cherof",
			discColor: "#9333ea",
			url: "https://raw.githubusercontent.com/InventivetalentDev/minecraft-assets/1.20.1/assets/minecraft/sounds/records/relic.ogg"
		}
	];

	let activeTrack = $state<Track>(playlist[0]);
</script>

<Modal isOpen={open} {onClose} title="Jukebox do Launcher — Discos e Sons">
	<div class="flex flex-col gap-4 text-xs">
		<p class="text-white/70 leading-relaxed">
			Escute os discos de música clássicos e pré-visualize trilhas sonoras de Resource Packs com onda sonora interativa diretamente no launcher antes de entrar no Minecraft.
		</p>

		<!-- Visualizer Card -->
		<AudioVisualizer url={activeTrack.url} title={`${activeTrack.title} — ${activeTrack.author}`} />

		<!-- Playlist Grid -->
		<div class="flex flex-col gap-2">
			<span class="text-[11px] font-bold text-white/50 uppercase tracking-wider">Discos Disponíveis</span>
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
				{#each playlist as track}
					<button
						type="button"
						class="flex items-center gap-3 p-2.5 rounded-xl border text-left transition-all cursor-pointer {activeTrack.id === track.id ? 'bg-[#caa97c]/15 border-[#caa97c]/50' : 'bg-bg-subtle border-white/5 hover:border-white/20'}"
						onclick={() => activeTrack = track}
					>
						<div 
							class="w-8 h-8 rounded-full flex items-center justify-center shrink-0 shadow-md"
							style="background-color: {track.discColor}25; border: 1px solid {track.discColor}60; color: {track.discColor};"
						>
							<Disc class="w-4 h-4 {activeTrack.id === track.id ? 'animate-spin' : ''}" />
						</div>

						<div class="min-w-0 flex-1">
							<h4 class="text-white font-bold text-xs truncate">{track.title}</h4>
							<p class="text-[10px] text-white/50 truncate">{track.author}</p>
						</div>

						{#if activeTrack.id === track.id}
							<span class="text-[10px] font-bold text-[#caa97c] px-2 py-0.5 rounded-full bg-[#caa97c]/20">
								Tocando
							</span>
						{/if}
					</button>
				{/each}
			</div>
		</div>

		<div class="flex items-center justify-end pt-2 border-t border-white/5">
			<Button variant="secondary" size="sm" onclick={onClose}>
				Fechar
			</Button>
		</div>
	</div>
</Modal>
