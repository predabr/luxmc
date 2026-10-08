<script lang="ts">
    import { roomPreparation } from '$lib/stores/roomPreparation.svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { button } from '$lib/components/ui/button';
    import { LoaderCircle, CheckCircle2, X } from 'lucide-svelte';
</script>
{#if roomPreparation.state.ownerId}
    <aside class="fixed bottom-6 left-1/2 z-[160] w-[min(24rem,calc(100vw-10rem))] -translate-x-1/2 rounded-3xl border border-brand-400/30 bg-bg-elevated p-5 shadow-elevated" aria-label={t('studio.prepare')}>
        <div class="flex items-center gap-3">{#if roomPreparation.state.busy}<LoaderCircle class="h-5 w-5 animate-spin text-brand-400 motion-reduce:animate-none" />{:else}<CheckCircle2 class="h-5 w-5 text-brand-400" />{/if}<h2 class="min-w-0 flex-1 truncate text-sm font-semibold">{roomPreparation.state.name || t('studio.prepare')}</h2><button type="button" disabled={roomPreparation.state.busy} aria-label={t('common.close')} class={button({variant:'ghost',size:'icon'})} onclick={()=>roomPreparation.dismiss()}><X class="h-4 w-4" /></button></div>
        <p role="status" class="mt-3 text-xs text-fg-muted">{roomPreparation.state.text}</p>
        <progress class="mt-3 h-1.5 w-full accent-brand-400" max="100" value={roomPreparation.state.percent} aria-label={t('studio.prepare')}></progress>
        {#if roomPreparation.state.error}<p role="alert" class="mt-3 text-xs text-danger">{roomPreparation.state.error}</p>{/if}
        {#if roomPreparation.state.profileId && !roomPreparation.state.busy}<a href={`/instances/${roomPreparation.state.profileId}`} class={button({variant:'secondary',size:'sm',class:'mt-3'})}>{t('studio.openPrepared')}</a>{/if}
    </aside>
{/if}
