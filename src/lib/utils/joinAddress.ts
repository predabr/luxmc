import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
export function parseJoinAddress(value: string): { host: string; port: number } {
	const raw = value.trim().replace(/^luxmc:\/\/join\//, "");
	if (!raw || /[\s/@?#\\]/.test(raw) || /^LUX-/i.test(raw)) throw new Error(uiText("ui.6f17ad9e00a98d14"));
	const match = raw.match(/^(\[[a-fA-F0-9:]+\]|[a-zA-Z0-9](?:[a-zA-Z0-9.-]*[a-zA-Z0-9])?)(?::(\d{1,5}))?$/);
	if (!match) throw new Error(uiText("ui.a289c518d41ca6e0"));
	const port = Number(match[2] || 25565);
	if (port < 1 || port > 65535) throw new Error(uiText("ui.fa29cbd6c8ed9b55"));
	const host = match[1];
    if (host.startsWith("[")) {
        try { new URL(`http://${host}:${port}`); } catch { throw new Error(uiText("ui.0d1d43716ebe17d3")); }
    } else if (!host.split(".").every(label => /^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$/.test(label)) || host.length > 253) {
        throw new Error(uiText("ui.6081e8d49e53c20f"));
    }
    return { host, port };
}

