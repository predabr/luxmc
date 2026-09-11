<script lang="ts">
  let { size = "md", color = "brand" }: { size?: "sm" | "md" | "lg"; color?: "brand" | "white" | "muted" } = $props();

  const sizes = { sm: "h-4 w-4", md: "h-6 w-6", lg: "h-8 w-8" };
  const colors = {
    brand: "border-brand-500",
    white: "border-white",
    muted: "border-fg-muted"
  };
</script>

<div class="animate-spin rounded-full border-2 border-t-transparent {sizes[size]} {colors[color]}" role="status">
  <span class="sr-only">Loading...</span>
</div>
