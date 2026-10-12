<script lang="ts">
    import { Crown, Gem, Sparkles, FlaskConical, HeartHandshake, Tag, X, Plus, Check } from 'lucide-svelte';
    import { translateUi } from '$lib/i18n/useTranslation.svelte';

    let { label, kind = 'tag', size = 'sm', action, disabled = false, onaction, order = 0 }: {
        label: string;
        kind?: 'owner' | 'plus' | 'pro' | 'tag';
        size?: 'sm' | 'md';
        action?: 'add' | 'remove';
        disabled?: boolean;
        onaction?: () => void;
        order?: number;
    } = $props();

    const identity = $derived.by(() => {
        if (kind === 'owner') return { icon: Crown, tone: 'warning' };
        if (kind === 'pro') return { icon: Gem, tone: 'brand-400' };
        if (kind === 'plus') return { icon: Sparkles, tone: 'info' };
        const name = label.normalize('NFD').replace(/[\u0300-\u036f]/g, '').trim().toLowerCase();
        if (['testador', 'tester', 'beta', 'beta tester'].includes(name)) return { icon: FlaskConical, tone: 'info' };
        if (['amigo', 'amiga', 'friend'].includes(name)) return { icon: HeartHandshake, tone: 'success' };
        if (['pro', 'luxmc pro'].includes(name)) return { icon: Gem, tone: 'brand-400' };
        if (['plus', 'luxmc plus'].includes(name)) return { icon: Sparkles, tone: 'info' };
        return { icon: Tag, tone: 'fg-muted' };
    });
    const badgeStyle = $derived(`--badge-accent: var(--${identity.tone}); --badge-order: ${order}`);
</script>

{#snippet content()}
    <span class="badge-symbol" aria-hidden="true"><identity.icon strokeWidth={1.8} /></span>
    <span class="badge-label">{label}</span>
    {#if action === 'remove'}<X class="badge-action" aria-hidden="true" />
    {:else if action === 'add'}<Plus class="badge-action" aria-hidden="true" />
    {:else if kind === 'owner'}<Check class="badge-seal" strokeWidth={2.6} aria-hidden="true" />{/if}
{/snippet}

{#if action}
    <button type="button" class="profile-badge font-sans profile-badge--{kind} profile-badge--{size} profile-badge--{action}" style={badgeStyle} {disabled} onclick={onaction} title={`${translateUi(action === 'remove' ? 'common.remove' : 'common.add')}: ${label}`} aria-label={action === 'add' ? `+ ${label}` : `${translateUi('common.remove')}: ${label}`}>
        {@render content()}
    </button>
{:else}
    <span class="profile-badge font-sans profile-badge--{kind} profile-badge--{size}" style={badgeStyle} title={label}>
        {@render content()}
    </span>
{/if}

<style>
    .profile-badge {
        position: relative;
        display: inline-flex;
        align-items: center;
        gap: 6px;
        max-inline-size: 100%;
        min-inline-size: 0;
        min-block-size: 25px;
        padding: 3px 8px 3px 4px;
        border: 1px solid rgb(var(--badge-accent) / .24);
        border-radius: 9px;
        background: linear-gradient(120deg, rgb(var(--badge-accent) / .1), rgb(var(--bg-elevated) / .92) 82%);
        color: rgb(var(--fg));
        box-shadow: inset 0 1px 0 rgb(var(--surface-shine) / .07), 0 2px 5px rgb(var(--shadow-color) / .05);
        vertical-align: middle;
        animation: badge-in 300ms cubic-bezier(.16,1,.3,1) both;
        animation-delay: calc(var(--badge-order) * 35ms);
        transition: border-color 180ms ease, background-color 180ms ease, translate 180ms ease;
    }
    .badge-symbol { display: grid; place-items: center; inline-size: 18px; block-size: 18px; flex-shrink: 0; border-radius: 5px; color: rgb(var(--badge-accent)); background: rgb(var(--badge-accent) / .12); }
    .badge-symbol :global(svg) { inline-size: 12px; block-size: 12px; }
    .badge-label { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; font-weight: 650; line-height: 1.35; letter-spacing: .015em; }
    .profile-badge--owner { border-color: rgb(var(--badge-accent) / .38); background: linear-gradient(115deg, rgb(var(--badge-accent) / .18), rgb(var(--bg-elevated) / .95)); }
    .profile-badge--pro { border-color: rgb(var(--badge-accent) / .38); }
    .profile-badge--owner .badge-symbol, .profile-badge--pro .badge-symbol { background: rgb(var(--badge-accent) / .19); }
    .profile-badge :global(.badge-seal) { inline-size: 11px; block-size: 11px; flex-shrink: 0; color: rgb(var(--badge-accent)); }
    .profile-badge :global(.badge-action) { inline-size: 12px; block-size: 12px; flex-shrink: 0; color: rgb(var(--fg-muted)); }
    .profile-badge--md { min-block-size: 30px; padding: 4px 10px 4px 5px; gap: 8px; border-radius: 10px; }
    .profile-badge--md .badge-symbol { inline-size: 21px; block-size: 21px; border-radius: 6px; }
    .profile-badge--md .badge-symbol :global(svg) { inline-size: 14px; block-size: 14px; }
    .profile-badge--md .badge-label { font-size: 12px; }
    .profile-badge--add { background: rgb(var(--bg-elevated) / .55); border-style: dashed; box-shadow: none; }
    button.profile-badge { cursor: pointer; min-block-size: 34px; }
    button.profile-badge:hover:not(:disabled) { translate: 0 -1px; border-color: rgb(var(--badge-accent) / .6); background-color: rgb(var(--badge-accent) / .08); }
    button.profile-badge:hover:not(:disabled) :global(.badge-action) { color: rgb(var(--badge-accent)); }
    button.profile-badge:focus-visible { outline: 2px solid rgb(var(--badge-accent)); outline-offset: 3px; }
    button.profile-badge:disabled { cursor: wait; opacity: .5; }
    @keyframes badge-in { from { opacity: 0; translate: 0 4px; scale: .96; } to { opacity: 1; translate: 0 0; scale: 1; } }
    :global(html.no-ui-motion) .profile-badge, :global(html.no-anim) .profile-badge { animation: none; transition: none; }
    :global(html.no-ui-motion) button.profile-badge:hover, :global(html.no-anim) button.profile-badge:hover { translate: none; }
    @media (prefers-reduced-motion: reduce) {
        :global(html:not(.force-ui-motion)) .profile-badge { animation: none; transition: none; }
        :global(html:not(.force-ui-motion)) button.profile-badge:hover { translate: none; }
    }
</style>
