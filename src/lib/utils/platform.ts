import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
export type LauncherPlatform = "windows" | "linux" | "macos" | "unknown";

export function detectPlatform(value: string): LauncherPlatform {
    if (/windows|win32|win64|winn?t/i.test(value)) return "windows";
    if (/macintosh|macos|mac os|darwin/i.test(value)) return "macos";
    if (/linux|ubuntu|debian|fedora|arch|mint|pop!|suse/i.test(value)) return "linux";
    return "unknown";
}

export function recommendMemory(totalMb: number): { minRamMb: number; maxRamMb: number } {
    if (!Number.isFinite(totalMb) || totalMb < 2048) throw new Error(uiText("ui.0b82e10896393c5f"));
    const budget = Math.min(totalMb * .5, totalMb - 2048, 8192);
    const maxRamMb = Math.max(1024, Math.floor(budget / 512) * 512);
    return { minRamMb: Math.min(1024, maxRamMb), maxRamMb };
}
