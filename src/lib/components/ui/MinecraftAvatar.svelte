<script lang="ts">
    import { lastSeenLabel } from "$lib/utils/lastSeen";
    import { minecraftUuid } from "$lib/api/skins";
	let { username, status = "offline", activity, lastSeen, class: className = "h-12 w-12" }: {
		username: string;
		status?: "online" | "in_game" | "offline" | "pending";
		activity?: string | null;
		lastSeen?: string | null;
		class?: string;
	} = $props();
	let fallback = $state(0);
    let resolvedUuid = $state<string | null>(null);
	$effect(() => { username; fallback = 0; resolvedUuid = null; });
    $effect(() => {
        if (fallback !== 2) return;
        let current = true;
        void minecraftUuid(username).then(uuid => { if (current) { resolvedUuid = uuid; if (!uuid) fallback = 3; } }).catch(() => { if (current) fallback = 3; });
        return () => { current = false; };
    });
	const source = $derived(fallback === 0 ? `https://mc-heads.net/avatar/${encodeURIComponent(username)}/64` : fallback === 1 ? `https://minotar.net/helm/${encodeURIComponent(username)}/64` : fallback === 2 && resolvedUuid ? `https://crafatar.com/avatars/${resolvedUuid}?size=64&overlay` : "/grass_head.png");
	const ring = $derived(status === "in_game" ? "ring-purple-500/80 shadow-lg shadow-purple-500/30" : status === "online" ? "ring-success/80 shadow-lg shadow-success/30" : status === "pending" ? "ring-warning/50" : "ring-border-strong/50");
	const label = $derived(status === "in_game" ? `Jogando ${activity || "Minecraft"}` : status === "online" ? "Online no Launcher" : status === "pending" ? "Solicitação pendente" : lastSeenLabel(lastSeen));
</script>

<div class="relative shrink-0 rounded-2xl bg-bg-subtle ring-2 ring-offset-2 ring-offset-bg-elevated {ring} {className}" title={label}>
	<img src={source} alt={`Skin de ${username || "Steve"}`} class="h-full w-full rounded-2xl object-cover [image-rendering:pixelated]" class:opacity-60={status === "offline"} class:grayscale={status === "offline"} loading="lazy" referrerpolicy="no-referrer" onerror={() => { if (fallback < 3) fallback += 1; }} />
	<span class="absolute -bottom-1 -right-1 h-3 w-3 rounded-full border-2 border-bg-elevated {status === 'online' ? 'bg-success motion-safe:animate-pulse' : status === 'in_game' ? 'bg-purple-500' : status === 'pending' ? 'bg-warning' : 'bg-fg-subtle'}"></span>
</div>
