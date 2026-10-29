<script lang="ts">
    import { tick } from 'svelte';
    import { Copy, Send, LoaderCircle, Users, Save } from 'lucide-svelte';
    import { translateUi as t } from '$lib/i18n/useTranslation.svelte';
    import { serverConsoleText, type ServerConsoleLine } from '$lib/api/localServers';
    import { toast } from '$lib/stores/toasts.svelte';
    import { button } from './serverButton';

    let { id, lines, available, send }: { id: string; lines: ServerConsoleLine[]; available: boolean; send: (text: string) => Promise<boolean> } = $props();
    let element = $state<HTMLDivElement>();
    let follow = $state(true); let command = $state(''); let busy = $state(false); let copying = $state(false); let error = $state('');
    const contentWarnings = $derived(lines.some(line => /Couldn't load tag|Block-attached entity at invalid position|Tried to load a block entity/.test(line.text)));
    const overloaded = $derived(lines.some(line => line.text.includes("Can't keep up!")));
    let history = $state<string[]>([]); let position = $state(0); let draft = '';
    $effect(() => { const count = lines.length; const last = lines.at(-1)?.sequence; if (follow && count && last) void tick().then(() => { if (element && follow) element.scrollTop = element.scrollHeight; }); });
    async function submit(text = command) {
        if (busy || !available || !text.trim()) return;
        busy = true;
        try { if (await send(text)) { history = [...history.filter(item => item !== text), text].slice(-40); position = history.length; command = ''; draft = ''; } }
        finally { busy = false; }
    }
    function navigate(event: KeyboardEvent) {
        if (!['ArrowUp', 'ArrowDown'].includes(event.key) || !history.length) return;
        event.preventDefault(); if (position === history.length) draft = command;
        position = Math.max(0, Math.min(history.length, position + (event.key === 'ArrowUp' ? -1 : 1)));
        command = position === history.length ? draft : history[position];
    }
    async function copyConsole() {
        if (copying) return; copying = true; error = '';
        try { const result = await serverConsoleText(id); await navigator.clipboard.writeText(result.text || lines.map(line => line.text).join('\n')); toast(t(result.truncated ? 'consoleTools.copiedTail' : 'localServer.copied'), 'success'); }
        catch (cause) { error = String(cause); }
        finally { copying = false; }
    }
</script>

<div class="flex min-h-full flex-col gap-4 p-5" data-server-console>
    <div class="flex flex-wrap items-center justify-between gap-3"><h3 class="font-semibold">{t('localServer.console')}</h3><div class="flex flex-wrap items-center gap-3"><label class="flex items-center gap-2 text-xs text-fg-muted"><input type="checkbox" bind:checked={follow} />{t('serverPanel.follow')}</label><button type="button" class={button({variant:'secondary',size:'sm'})} onclick={copyConsole} disabled={copying}><Copy class="h-4 w-4" />{t('consoleTools.copy')}</button></div></div>
    {#if contentWarnings}<p class="rounded-xl border border-warning/30 bg-warning/5 p-3 text-xs leading-6 text-warning">{t('serverLogContext.content')}</p>{/if}
    {#if overloaded}<p class="rounded-xl border border-warning/30 bg-warning/5 p-3 text-xs leading-6 text-warning">{t('serverLogContext.load')}</p>{/if}
    {#if error}<p role="alert" class="text-sm text-danger">{error}</p>{/if}
    <div bind:this={element} role="log" aria-label={t('localServer.console')} aria-live="off" onpointerdown={() => follow = false} class="console-output custom-scrollbar min-h-64 flex-1 select-text overflow-auto rounded-2xl border border-border bg-bg-subtle p-4 font-mono text-xs leading-6" style="max-height:440px">
        {#if !lines.length}<p class="text-fg-muted">{t('localServer.consoleEmpty')}</p>{/if}
        {#each lines as line (line.sequence)}<div class="whitespace-pre-wrap break-words {line.text.includes('/ERROR]') ? 'text-danger' : line.stream === 'stderr' || line.text.includes('/WARN]') ? 'text-warning' : line.stream === 'command' || line.stream === 'response' ? 'text-brand-400' : 'text-fg-muted'}">{line.stream === 'command' ? '> ' : ''}{line.text}</div>{/each}
    </div>
    <div class="flex flex-wrap gap-2"><button type="button" class={button({variant:'secondary',size:'sm'})} disabled={busy || !available} onclick={() => submit('list')}><Users class="h-4 w-4" />{t('consoleTools.list')}</button><button type="button" class={button({variant:'secondary',size:'sm'})} disabled={busy || !available} onclick={() => submit('save-all flush')}><Save class="h-4 w-4" />{t('localServer.saveWorld')}</button></div>
    <form class="flex gap-3" onsubmit={event => { event.preventDefault(); void submit(); }}><input class="min-h-12 min-w-0 flex-1 rounded-xl border border-border bg-bg-subtle px-4 font-mono text-sm text-fg focus-visible:outline focus-visible:outline-brand-400" aria-label={t('localServer.command')} placeholder={t('localServer.commandHint')} maxlength="1024" bind:value={command} onkeydown={navigate} disabled={busy || !available} /><button type="submit" aria-label={t('localServer.send')} class={button({variant:'primary'})} disabled={busy || !available || !command.trim()}>{#if busy}<LoaderCircle class="h-4 w-4 animate-spin" />{:else}<Send class="h-4 w-4" />{/if}</button></form>
    <p class="text-xs leading-6 text-fg-muted">{t(available ? 'consoleTools.help' : 'consoleTools.offline')}</p>
</div>

<style>
    .console-output,.console-output :global(*) { user-select:text; -webkit-user-select:text; cursor:text; }
</style>
