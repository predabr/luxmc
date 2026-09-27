<script lang="ts">
    import Modal from "./Modal.svelte";
    import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
    let { open, onClose, profileId }: { open: boolean; onClose: () => void; profileId?: string } = $props();
</script>
<Modal isOpen={open} {onClose} title="Jogar com amigos" maxWidth="max-w-4xl">
    <MeshPanel {profileId} />
</Modal>
