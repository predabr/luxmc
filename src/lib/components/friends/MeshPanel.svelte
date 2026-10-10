<script lang="ts">
    import RoomPanel from "./RoomPanel.svelte";
    let { profileId: _profileId }: { profileId?: string } = $props();
</script>
<RoomPanel />
