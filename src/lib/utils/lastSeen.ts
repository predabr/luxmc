import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
export function lastSeenLabel(value: string | null | undefined, now: number = Date.now()): string {
    if (!value) return "Offline";
    const timestamp = Date.parse(value);
    if (!Number.isFinite(timestamp)) return "Offline";
    const minutes = Math.max(0, Math.floor((now - timestamp) / 60000));
    if (minutes < 1) return uiText("ui.25ab1ecd45e65586");
    if (minutes < 60) return uiText("ui.79564aca91f78230", {arg0: (minutes)});
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return uiText("ui.d31dc55460d3e0df", {arg0: (hours), arg1: (hours === 1 ? "hora" : "horas")});
    const days = Math.floor(hours / 24);
    return uiText("ui.d31dc55460d3e0df", {arg0: (days), arg1: (days === 1 ? "dia" : "dias")});
}
