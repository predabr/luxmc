<script lang="ts">
    import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
    import type { TunnelMember } from "$lib/api/tunnel";
    let { member }: { member: TunnelMember } = $props();
    let failed = $state(false);
    const avatar = $derived(member.avatarUrl);
    $effect(() => { avatar; failed = false; });
</script>

{#if member.avatarUrl && !failed}
    <img src={member.avatarUrl} alt={`Avatar de ${member.username}`} class="h-12 w-12 shrink-0 rounded-xl border border-border object-cover" referrerpolicy="no-referrer" onerror={() => failed = true} />
{:else}
    <MinecraftAvatar username={member.username} status="online" class="h-12 w-12" />
{/if}
