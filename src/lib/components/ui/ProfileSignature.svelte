<script lang="ts">
    import type { Snippet } from 'svelte';
    import { defaultProfileStyle, type ProfileStyle } from '$lib/utils/profileStyle';
    import '$lib/styles/profileSignature.css';
    let { signature = defaultProfileStyle, children, class: className = '' }: { signature?: ProfileStyle; children: Snippet; class?: string } = $props();
</script>

<div class="profile-signature {className}" data-profile-accent={signature.accent} data-profile-frame={signature.frame} data-profile-effect={signature.effect} data-profile-layout={signature.layout}>
    {@render children()}
</div>
