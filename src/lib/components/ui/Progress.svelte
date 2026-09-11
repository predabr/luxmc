<script lang="ts">
  let {
    value = 0,
    max = 100,
    class: klass = "",
    showLabel = false,
    variant = "brand"
  }: {
    value?: number;
    max?: number;
    class?: string;
    showLabel?: boolean;
    variant?: "brand" | "success" | "warning" | "danger";
  } = $props();

  let percentage = $derived(Math.min(100, Math.max(0, (value / max) * 100)));
  const variants = {
    brand: "bg-brand-500",
    success: "bg-success",
    warning: "bg-warning",
    danger: "bg-danger"
  };
</script>

<div class="relative {klass}">
  <div class="h-2 overflow-hidden rounded-full bg-bg-subtle">
    <div
      class="h-full rounded-full transition-all duration-300 ease-out {variants[variant]}"
      style="width: {percentage}%"
    ></div>
  </div>
  {#if showLabel}
    <span class="mt-1 text-xs text-fg-muted">{Math.round(percentage)}%</span>
  {/if}
</div>
