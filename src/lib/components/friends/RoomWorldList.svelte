<script lang="ts">
    import { Copy, RadioTower, ShieldCheck } from "lucide-svelte";
    import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import type { TunnelStatus } from "$lib/api/tunnel";
    import { button } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    import { profiles } from '$lib/stores/profiles.svelte';
    import { account } from '$lib/stores/account.svelte';
    import { instanceCompatibility, type Compatibility } from '$lib/api/experience';
    import { doctorInstanceReadiness } from '$lib/api/doctor';
    import GlassSelect from '$lib/components/ui/GlassSelect.svelte';
    const phaseLabels={idle:'studio.phase.idle',planning:'studio.phase.planning',downloading:'studio.phase.downloading',validating:'studio.phase.validating',ready:'studio.phase.ready',failed:'studio.phase.failed'};
    let selected = $state(profiles.activeId || '');
    $effect(() => { if (!profiles.list.some(profile => profile.id === selected)) selected=(profiles.list.some(profile => profile.id === profiles.activeId) ? profiles.activeId : null) || profiles.list[0]?.id || ''; });
    let local = $state<Compatibility | null>(null);
    let checking = $state(false);
    let checkError = $state('');
    let warnings = $state<string[]>([]);
    async function check() {
        if (checking || !selected) return;
        const id = selected; checking=true; checkError=''; local=null;
        try { const [compatibility,readiness] = await Promise.all([instanceCompatibility(id),doctorInstanceReadiness(id)]); if (id !== selected) return; local=compatibility; warnings=[...readiness.blockers,...readiness.warnings]; }
        catch(error) { checkError=String(error); } finally { checking=false; }
    }
    let { room }: { room: TunnelStatus } = $props();
    async function copy(address: string) {
        try { await navigator.clipboard.writeText(address); toast(uiText("multiplayer.addressCopied"), "success"); }
        catch (error) { toast(String(error), "error"); }
    }
</script>

<section class="surface-glass space-y-4 p-6">
    <div><h2 class="flex items-center gap-2 text-lg font-semibold text-fg"><RadioTower class="h-5 w-5 text-success" />{uiText("multiplayer.worldsTitle")}</h2><p class="mt-2 text-sm text-fg-muted">{uiText("multiplayer.worldsHelp")}</p></div>
    <div class="flex flex-wrap gap-3"><GlassSelect bind:value={selected} disabled={checking} options={profiles.list.map(profile=>({value:profile.id,label:profile.name}))} label={uiText('workshop.instance')} onchange={()=>{local=null;warnings=[];checkError='';}} /><button type="button" disabled={checking || !selected} class={button({variant:'secondary',size:'sm'})} onclick={check}><ShieldCheck class="h-4 w-4" />{checking ? uiText('common.loading') : uiText('workshop.preflight')}</button></div>
    {#if checkError}<p role="alert" class="text-xs text-danger">{checkError}</p>{/if}
    <div class="grid gap-2 sm:grid-cols-2">{#each room.members as member (member.id)}<div class="rounded-xl border border-border bg-bg-elevated p-3"><p class="truncate text-sm font-medium">{member.username}</p><p class="mt-1 text-xs text-fg-muted">{uiText(phaseLabels[member.preparation?.phase || 'idle'])}{#if member.preparation} · {member.preparation.percent}%{/if}</p>{#if member.preparation}<progress class="mt-2 h-1 w-full accent-brand-400" aria-label={member.username} value={member.preparation.percent} max="100"></progress>{/if}</div>{/each}</div>
    {#if local}<div role="status" class="space-y-2 rounded-2xl border border-border bg-bg-elevated p-4"><p class="text-xs text-fg-muted">{uiText('workshop.connection')}: {room.transport} · {room.pingMs === null ? uiText('workshop.unknown') : `${room.pingMs} ms`}</p><p class="text-xs text-fg-muted">{uiText('workshop.auth')}: {account.value?.id.startsWith('luxmc:') || !account.value?.minecraftToken ? uiText('workshop.authHelp') : account.value.expiresAt > 0 && account.value.expiresAt < Date.now() ? uiText('workshop.authExpired') : uiText('workshop.authPresent')}</p>{#each warnings as warning}<p class="text-xs text-warning">{warning}</p>{/each}</div>{/if}
    {#each room.worlds ?? [] as world (world.ownerId)}
        <div class="flex flex-wrap items-center gap-4 rounded-2xl border border-border bg-bg/30 p-4">
            <div class="min-w-0 flex-1"><p class="font-semibold text-fg">{world.ownerUsername}</p><p class="mt-1 break-words text-sm text-fg-muted">{world.motd.replace(/§./g, "")}</p></div>
            {#if world.localAddress}<code class="select-text text-sm text-success">{world.localAddress}</code><button type="button" class={button({ variant: "secondary", size: "sm" })} onclick={() => copy(world.localAddress!)}><Copy class="h-4 w-4" />{uiText("common.copy")}</button>{:else}<span class="text-xs text-fg-muted">{uiText("multiplayer.yourWorld")}</span>{/if}
        </div>
        {#if local && world.localAddress}<div class="grid gap-2 rounded-xl border border-border bg-bg-elevated p-4 text-xs sm:grid-cols-3">{#each [{label:uiText('workshop.version'),known:!!world.compatibility?.mcVersion,equal:world.compatibility?.mcVersion===local.mcVersion},{label:uiText('workshop.loader'),known:!!world.compatibility?.loader,equal:world.compatibility?.loader===local.loader},{label:uiText('workshop.mods'),known:!!world.compatibility?.modFingerprint,equal:world.compatibility?.modFingerprint===local.modFingerprint}] as item}<div><p class="text-fg-muted">{item.label}</p><p class="mt-1 {item.known ? item.equal ? 'text-success' : 'text-warning' : 'text-fg-muted'}">{!item.known ? uiText('workshop.unknown') : item.equal ? uiText('workshop.match') : uiText('workshop.mismatch')}</p></div>{/each}<p class="col-span-full mt-2 leading-relaxed text-fg-muted">{uiText('workshop.modsHelp')}</p></div>{/if}
    {:else}<p class="rounded-xl border border-border p-4 text-sm text-fg-muted">{uiText("multiplayer.waitingAnyWorld")}</p>{/each}
</section>
