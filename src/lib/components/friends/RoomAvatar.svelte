<script lang="ts">
    import MinecraftAvatar from "$lib/components/ui/MinecraftAvatar.svelte";
    import type { TunnelMember } from "$lib/api/tunnel";
    import type { ProfileStyle } from '$lib/utils/profileStyle';
    import ProfileSignature from '$lib/components/ui/ProfileSignature.svelte';
    let { member }: { member: TunnelMember & {profileStyle?:ProfileStyle} } = $props();
    let failed = $state(false);
    const avatar = $derived(member.avatarUrl);
    $effect(() => { avatar; failed = false; });
</script>

{#if member.avatarUrl && !failed}
    <ProfileSignature signature={member.profileStyle} class="contents"><div data-profile-avatar class="h-12 w-12 shrink-0 rounded-xl"><img src={member.avatarUrl} alt={`Avatar de ${member.username}`} class="h-full w-full rounded-xl border border-border object-cover" referrerpolicy="no-referrer" onerror={() => failed = true} /></div></ProfileSignature>
{:else}
    <MinecraftAvatar username={member.username} profileStyle={member.profileStyle} status="online" class="h-12 w-12" />
{/if}
