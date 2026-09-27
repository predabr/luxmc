<script lang="ts">
    import { onMount } from "svelte";
    import { open } from "@tauri-apps/plugin-dialog";
    import { Archive, Activity, Bug, Download, RotateCcw, ShieldCheck, Sparkles } from "lucide-svelte";
    import Button from "$lib/components/ui/Button.svelte";
    import { appState } from "$lib/stores/app.svelte";
    import { profiles } from "$lib/stores/profiles.svelte";
    import { toast } from "$lib/stores/toasts.svelte";
    import { listenGameTelemetry } from "$lib/api/events";
    import {
        instanceBenchmarkImport, instanceBenchmarksList,
        instanceCapsuleCreate, instanceCapsuleRestore, instanceCapsulesList,
        instanceIsolationReport, instanceIsolationRestore, instanceIsolationStart, instanceIsolationStatus,
        type BenchmarkResult, type CapsuleInfo, type IsolationState
    } from "$lib/api/instanceLab";

    let { profileId }: { profileId: string } = $props();
    let isolation = $state<IsolationState | null>(null);
    let capsules = $state<CapsuleInfo[]>([]);
    let benchmarks = $state<BenchmarkResult[]>([]);
    let capsuleLabel = $state("");
    let benchmarkLabel = $state("");
    let busy = $state(false);
    let loading = $state(true);

    const comparison = $derived(benchmarks.length >= 2 ? {
        fps: benchmarks[0].averageFps - benchmarks[1].averageFps,
        low: benchmarks[0].lowOnePercentFps - benchmarks[1].lowOnePercentFps,
        drops: benchmarks[0].fpsDrops - benchmarks[1].fpsDrops
    } : null);

    function date(value: string) {
        return new Date(value).toLocaleString("pt-BR", { dateStyle: "short", timeStyle: "short" });
    }

    async function refresh() {
        loading = true;
        try {
            [isolation, capsules, benchmarks] = await Promise.all([
                instanceIsolationStatus(profileId), instanceCapsulesList(profileId), instanceBenchmarksList(profileId)
            ]);
        } catch (error) {
            toast(`Não foi possível carregar o laboratório: ${String(error)}`, "error");
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        void refresh();
        let disposed = false;
        let unlisten: (() => void) | undefined;
        void listenGameTelemetry(summary => {
            if (disposed || summary.profileId !== profileId || !isolation || isolation.phase !== "testing") return;
            if (!summary.cleanExit && ![130, 137, 143].includes(summary.exitCode)) {
                void report(true, true);
            }
        }).then(stop => {
            if (disposed) stop(); else unlisten = stop;
        }).catch(() => {});
        return () => { disposed = true; unlisten?.(); };
    });

    async function startIsolation() {
        busy = true;
        try {
            isolation = await instanceIsolationStart(profileId);
            toast("Primeiro grupo de mods desativado. Inicie o jogo para testar.", "info");
        } catch (error) {
            toast(`Não foi possível iniciar o diagnóstico: ${String(error)}`, "error");
        } finally { busy = false; }
    }

    async function report(crashed: boolean, automatic = false) {
        if (busy || !isolation || isolation.phase !== "testing") return;
        busy = true;
        try {
            isolation = await instanceIsolationReport(profileId, crashed);
            if (isolation.phase === "found") toast(`Mod suspeito isolado: ${isolation.suspects[0]}. Todos os mods foram restaurados.`, "success");
            else toast(`${automatic ? "Crash detectado. " : ""}Próximo grupo preparado para teste.`, "info");
        } catch (error) {
            toast(`Diagnóstico interrompido: ${String(error)}`, "error");
        } finally { busy = false; }
    }

    async function restoreMods() {
        busy = true;
        try {
            await instanceIsolationRestore(profileId);
            isolation = null;
            toast("Todos os mods voltaram ao estado anterior ao diagnóstico.", "success");
        } catch (error) {
            toast(`Não foi possível restaurar os mods: ${String(error)}`, "error");
        } finally { busy = false; }
    }

    async function createCapsule() {
        busy = true;
        try {
            const created = await instanceCapsuleCreate(profileId, capsuleLabel);
            capsules = [created, ...capsules];
            capsuleLabel = "";
            toast("Cápsula criada com mundos, mods e configurações.", "success");
        } catch (error) {
            toast(`Falha ao criar cápsula: ${String(error)}`, "error");
        } finally { busy = false; }
    }

    async function restoreCapsule(filename: string) {
        if (!window.confirm("Restaurar esta cápsula? O estado atual será salvo automaticamente em outra cápsula antes da troca.")) return;
        busy = true;
        try {
            const previous = await instanceCapsuleRestore(profileId, filename);
            capsules = [previous, ...capsules];
            await profiles.refresh();
            toast("Cápsula restaurada. O estado anterior também está disponível na lista.", "success");
        } catch (error) {
            toast(`Falha ao restaurar cápsula: ${String(error)}`, "error");
        } finally { busy = false; }
    }

    async function importBenchmark() {
        try {
            const selected = await open({ multiple: false, filters: [{ name: "Log de desempenho MangoHud", extensions: ["csv"] }] });
            if (typeof selected !== "string") return;
            busy = true;
            const result = await instanceBenchmarkImport(profileId, benchmarkLabel, selected);
            benchmarks = [result, ...benchmarks];
            benchmarkLabel = "";
            toast("Medição de FPS adicionada ao laboratório.", "success");
        } catch (error) {
            toast(`CSV de desempenho inválido: ${String(error)}`, "error");
        } finally { busy = false; }
    }
</script>

<div class="space-y-5" aria-label="Laboratório da instância">
    {#if loading}
        <div class="rounded-2xl border border-fg/10 bg-bg-elevated p-8 text-center text-sm text-fg-muted">Carregando laboratório…</div>
    {:else}
        <div class="grid gap-5 xl:grid-cols-2">
            <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
                <div class="flex items-center gap-3"><Bug class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">Isolador de mods</h3><p class="text-xs text-fg-muted">Testa metades dos mods sem apagar nenhum arquivo.</p></div></div>
                {#if isolation}
                    {#if isolation.phase === "found"}
                        <div class="rounded-xl border border-success/30 bg-success/10 p-4 text-sm text-fg"><ShieldCheck class="inline h-4 w-4 text-success mr-1" />Suspeito: <strong>{isolation.suspects[0]}</strong>. Os mods foram restaurados para confirmação.</div>
                    {:else}
                        <div class="rounded-xl border border-brand-500/30 bg-brand-500/10 p-4 text-sm text-fg">Rodada {isolation.round}: {isolation.trialDisabled.length} mods temporariamente desativados. Use <strong>Jogar</strong> acima, depois informe se o jogo abriu. Um crash com código de erro é registrado automaticamente.</div>
                        <div class="flex flex-wrap gap-2">
                            <Button variant="primary" disabled={busy || appState.isGameRunning} onclick={() => report(false)}>Abriu sem crash</Button>
                            <Button variant="danger" disabled={busy || appState.isGameRunning} onclick={() => report(true)}>Crash persistiu</Button>
                        </div>
                    {/if}
                    <Button variant="secondary" disabled={busy || appState.isGameRunning} onclick={restoreMods}><RotateCcw class="h-4 w-4" /> Restaurar todos os mods</Button>
                {:else}
                    <p class="text-xs leading-relaxed text-fg-muted">Indicado quando o mesmo crash aparece a cada inicialização. Dependências entre mods podem exigir uma confirmação final.</p>
                    <Button variant="primary" disabled={busy || appState.isGameRunning} onclick={startIsolation}>Iniciar diagnóstico reversível</Button>
                {/if}
            </section>

            <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
                <div class="flex items-center gap-3"><Archive class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">Cápsulas da instância</h3><p class="text-xs text-fg-muted">Mundos, mods, configs, shaders, recursos e opções juntos.</p></div></div>
                <div class="flex flex-wrap gap-2">
                    <input class="luxmc-control min-w-0 flex-1" aria-label="Nome da cápsula" placeholder="Nome da cápsula" maxlength="48" bind:value={capsuleLabel} />
                    <Button variant="primary" disabled={busy || appState.isGameRunning || !!isolation} onclick={createCapsule}><Archive class="h-4 w-4" /> Criar cápsula</Button>
                </div>
                {#if capsules.length === 0}<p class="text-xs text-fg-muted">Nenhuma cápsula criada.</p>{/if}
                <div class="max-h-60 space-y-2 overflow-y-auto">
                    {#each capsules as capsule (capsule.filename)}
                        <div class="flex items-center justify-between gap-3 rounded-xl border border-fg/10 bg-bg-subtle/70 p-3">
                            <div class="min-w-0"><div class="truncate text-sm font-semibold text-fg">{capsule.label}</div><div class="text-xs text-fg-muted">{date(capsule.createdAt)} · {(capsule.sizeBytes / 1048576).toFixed(1)} MB</div></div>
                            <Button variant="secondary" size="sm" disabled={busy || appState.isGameRunning || !!isolation} onclick={() => restoreCapsule(capsule.filename)}><RotateCcw class="h-3.5 w-3.5" /> Restaurar</Button>
                        </div>
                    {/each}
                </div>
            </section>
        </div>

        <section class="rounded-2xl border border-fg/10 bg-bg-elevated/90 p-5 shadow-soft space-y-4">
            <div class="flex items-center gap-3"><Activity class="h-5 w-5 text-brand-400" /><div><h3 class="font-bold text-fg">Laboratório de desempenho</h3><p class="text-xs text-fg-muted">Compare FPS médio, 1% baixo e quedas entre duas sessões.</p></div></div>
            <p class="text-xs leading-relaxed text-fg-muted">No Linux, grave uma sessão com MangoHud (Shift esquerdo + F2), jogue no mesmo mundo e importe o CSV. Faça outra medição depois de alterar mods ou shaders para comparar em condições semelhantes.</p>
            <div class="flex flex-wrap gap-2">
                <input class="luxmc-control min-w-0 flex-1" aria-label="Nome da medição" placeholder="Ex.: antes do shader" maxlength="48" bind:value={benchmarkLabel} />
                <Button variant="primary" disabled={busy} onclick={importBenchmark}><Download class="h-4 w-4" /> Importar CSV MangoHud</Button>
            </div>
            {#if comparison}
                <div class="rounded-xl border border-brand-500/20 bg-brand-500/10 p-4 text-sm text-fg flex flex-wrap gap-x-6 gap-y-1">
                    <span>FPS médio: <strong>{comparison.fps >= 0 ? '+' : ''}{comparison.fps.toFixed(1)}</strong></span>
                    <span>1% baixo: <strong>{comparison.low >= 0 ? '+' : ''}{comparison.low.toFixed(1)}</strong></span>
                    <span>Quedas: <strong>{comparison.drops >= 0 ? '+' : ''}{comparison.drops}</strong></span>
                    <span class="text-xs text-fg-muted">última medição vs. anterior</span>
                </div>
            {/if}
            {#if benchmarks.length === 0}<p class="text-xs text-fg-muted">Importe a primeira medição para iniciar uma comparação.</p>{/if}
            <div class="grid gap-2 md:grid-cols-2">
                {#each benchmarks as result (result.id)}
                    <div class="rounded-xl border border-fg/10 bg-bg-subtle/70 p-3">
                        <div class="flex items-center gap-2"><Sparkles class="h-4 w-4 text-brand-400" /><strong class="truncate text-sm text-fg">{result.label}</strong></div>
                        <div class="mt-1 text-xs text-fg-muted">{date(result.createdAt)} · {result.samples} amostras</div>
                        <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-fg"><span>{result.averageFps.toFixed(1)} FPS médios</span><span>{result.lowOnePercentFps.toFixed(1)} FPS no 1% baixo</span><span>{result.fpsDrops} quedas</span></div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>
