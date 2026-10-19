<script lang="ts">
    import Modal from '$lib/components/ui/Modal.svelte';
    import { button } from '$lib/components/ui/button';
    import { planPrompt } from '$lib/stores/planPrompt.svelte';
    import { account } from '$lib/stores/account.svelte';
    import { openMembershipPortal } from '$lib/api/membership';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { Sparkles, ArrowUpRight, LoaderCircle } from 'lucide-svelte';
    let busy = $state(false);
    let error = $state('');
    function close() { planPrompt.close(); error = ''; }
    async function plans() {
        if (busy || !account.value?.id) return;
        busy = true; error = '';
        try { await openMembershipPortal(account.value.id); close(); }
        catch { error = t('planPrompt.openError'); }
        finally { busy = false; }
    }
</script>

<Modal isOpen={Boolean(planPrompt.feature)} onClose={close} title={t('planPrompt.title')}>
    <div class="mb-5 flex h-14 w-14 items-center justify-center rounded-2xl border border-brand-400/25 bg-brand-500/10 text-brand-400"><Sparkles class="h-6 w-6" /></div>
    <p class="font-semibold">{planPrompt.feature}</p>
    <p class="mt-3 text-sm leading-6 text-fg-muted">{t('planPrompt.help')}</p>
    <p class="mt-3 text-xs leading-5 text-fg-muted">{t('planPrompt.wallpaper')}</p>
    {#if error}<p role="alert" class="mt-4 text-sm text-danger">{error}</p>{/if}
    <div class="mt-6 flex flex-wrap justify-end gap-2">
        <button type="button" class={button({variant:'ghost'})} onclick={close}>{t('planPrompt.later')}</button>
        <button type="button" class={button({variant:'primary'})} disabled={busy || !account.value} onclick={plans}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<ArrowUpRight class="h-4 w-4" />{/if}{t('planPrompt.plans')}</button>
    </div>
</Modal>
