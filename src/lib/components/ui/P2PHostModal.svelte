<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import Modal from "./Modal.svelte";
    import MeshPanel from "$lib/components/friends/MeshPanel.svelte";
    let { open, onClose, profileId }: { open: boolean; onClose: () => void; profileId?: string } = $props();
</script>
<Modal isOpen={open} {onClose} title={uiText("ui.4cab082a07901725")} maxWidth="max-w-4xl">
    <MeshPanel {profileId} />
</Modal>
