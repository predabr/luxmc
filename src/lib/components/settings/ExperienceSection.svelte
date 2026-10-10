<script lang="ts">
    import { settings, type AppSettings } from '$lib/stores/settings.svelte';
    import { schedulePersist } from '$lib/stores/persistence.svelte';
    import { Sparkles, Image, Zap } from 'lucide-svelte';
    function update(value: Partial<AppSettings>) { settings.patch(value); schedulePersist(); }
</script>

<section class="space-y-5 rounded-2xl border border-border bg-bg-subtle p-5" aria-label="Interface e carregamento">
    <div class="flex items-center gap-3"><Sparkles class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">Interface e carregamento</h3><p class="mt-1 text-xs text-fg-muted">Escolha como o Luxmc prepara e apresenta sua biblioteca.</p></div></div>
    <div class="grid gap-4 sm:grid-cols-2">
        <label class="space-y-2 text-sm text-fg"><span>Estilo das animações</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.motionStyle ?? 'expressive'} onchange={event => update({ motionStyle: event.currentTarget.value as AppSettings['motionStyle'] })}><option value="expressive">Expressivo · entrada em sequência</option><option value="subtle">Discreto · movimentos curtos</option></select></label>
        <label class="space-y-2 text-sm text-fg"><span>Velocidade das transições</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.motionSpeed ?? 'balanced'} onchange={event => update({ motionSpeed: event.currentTarget.value as AppSettings['motionSpeed'] })}><option value="balanced">Suave</option><option value="fast">Rápida</option></select></label>
        <label class="space-y-2 text-sm text-fg"><span class="flex items-center gap-2"><Image class="h-4 w-4" />Qualidade dos GIFs</span><select class="w-full rounded-xl border border-border bg-bg-elevated p-3" value={settings.value.gifQuality ?? 'high'} onchange={event => update({ gifQuality: event.currentTarget.value as AppSettings['gifQuality'] })}><option value="high">Alta · melhor nitidez</option><option value="balanced">Equilibrada · menor processamento</option></select><small class="block text-fg-muted">Aplicada às próximas importações. A compressão preserva os quadros e a duração; GIFs grandes podem ser otimizados como WebP animado.</small></label>
        <label class="flex items-center justify-between gap-4 rounded-xl border border-border p-4 text-sm text-fg"><span><span class="flex items-center gap-2"><Zap class="h-4 w-4" />Preparar capas antecipadamente</span><small class="mt-2 block leading-relaxed text-fg-muted">Carrega os ícones da instância em segundo plano antes de você rolar a lista.</small></span><input type="checkbox" class="h-5 w-5 shrink-0 accent-brand-500" checked={settings.value.preloadContentIcons !== false} onchange={event => update({ preloadContentIcons: event.currentTarget.checked })} /></label>
    </div>
</section>
