import type { DeepLinkAction } from "$lib/utils/deepLink";

export const deepLinks = $state<{
    install: Extract<DeepLinkAction, { kind: "install" }> | null;
    skin: Extract<DeepLinkAction, { kind: "skin" }> | null;
}>({ install: null, skin: null });
