<script lang="ts">
    import VirtualLanPanel from "./VirtualLanPanel.svelte";
    let { profileId: _profileId }: { profileId?: string } = $props();
</script>
<VirtualLanPanel />