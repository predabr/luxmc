<script lang="ts">
  let { 
    class: klass = "",
    variant = "text",
    width,
    height
  }: {
    class?: string;
    variant?: "text" | "circular" | "rectangular";
    width?: string;
    height?: string;
  } = $props();
  
  const baseClass = "animate-pulse bg-bg-subtle";
  const variants = {
    text: "rounded h-4",
    circular: "rounded-full",
    rectangular: "rounded-xl"
  };
</script>

<div 
  class="{baseClass} {variants[variant]} {klass}"
  style:width={width}
  style:height={height}
></div>
