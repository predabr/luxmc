<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
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
  const variants = $derived({
    text: uiText("ui.c37976ec4c559578"),
    circular: "rounded-full",
    rectangular: "rounded-xl"
  });
</script>

<div 
  class="{baseClass} {variants[variant]} {klass}"
  style:width={width}
  style:height={height}
></div>
