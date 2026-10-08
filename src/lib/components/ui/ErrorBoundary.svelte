<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
  import type { Snippet } from "svelte";
  
  let { children, fallback }: { children: Snippet; fallback?: Snippet } = $props();
  let error = $state<Error | null>(null);
  
  function handleError(e: ErrorEvent) {
    if (!e.error) return;
    if (e.target && e.target !== window && (e.target as HTMLElement).tagName) return;
    error = e.error;
    console.error("ErrorBoundary caught:", e.error);
  }

  $effect(() => {
    window.addEventListener("error", handleError as EventListener);
    return () => window.removeEventListener("error", handleError as EventListener);
  });
</script>

<svelte:boundary>
{#if error}
  {#if fallback}
    {@render fallback()}
  {:else}
    <div class="flex flex-col items-center justify-center gap-4 p-8 text-center">
      <div class="rounded-full bg-danger/10 p-4">
        <svg class="h-8 w-8 text-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
      </div>
      <h3 class="text-lg font-semibold text-fg">{uiText("ui.8851485f672f4ea0")}</h3>
      <p class="text-sm text-fg-muted">{error.message}</p>
      <button 
        class={launcherButton({ variant: "primary", size: "sm", class: "" })}
        onclick={() => error = null}
      >
        {uiText("ui.b9e10688be012d8b")}
      </button>
    </div>
  {/if}
{:else}
  {@render children()}
{/if}
{#snippet failed(cause, reset)}
  <div role="alert" class="flex min-h-64 flex-col items-center justify-center gap-4 rounded-2xl border border-danger/20 bg-bg-elevated p-8 text-center">
    <h3 class="text-lg font-semibold text-fg">{uiText("ui.8851485f672f4ea0")}</h3>
    <p class="max-w-xl break-words text-sm text-fg-muted">{cause instanceof Error ? cause.message : String(cause)}</p>
    <div class="flex flex-wrap justify-center gap-3">
      <button type="button" class={launcherButton({variant:'primary'})} onclick={() => { error = null; reset(); }}>{uiText("ui.b9e10688be012d8b")}</button>
      <button type="button" class={launcherButton({variant:'secondary'})} onclick={() => window.location.reload()}>{uiText('recovery.reloadInterface')}</button>
    </div>
  </div>
{/snippet}
</svelte:boundary>
