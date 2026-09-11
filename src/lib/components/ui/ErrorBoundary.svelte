<script lang="ts">
  import type { Snippet } from "svelte";
  
  let { children, fallback }: { children: Snippet; fallback?: Snippet } = $props();
  let error = $state<Error | null>(null);
  
  function handleError(e: ErrorEvent) {
    error = e.error;
    console.error("ErrorBoundary caught:", e.error);
  }

  $effect(() => {
    window.addEventListener("error", handleError as EventListener);
    return () => window.removeEventListener("error", handleError as EventListener);
  });
</script>

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
      <h3 class="text-lg font-semibold text-fg">Algo deu errado</h3>
      <p class="text-sm text-fg-muted">{error.message}</p>
      <button 
        class="rounded-xl bg-brand-500 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-600 transition-colors"
        onclick={() => error = null}
      >
        Tentar novamente
      </button>
    </div>
  {/if}
{:else}
  {@render children()}
{/if}
