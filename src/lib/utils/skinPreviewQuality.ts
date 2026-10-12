export type SkinPreviewQuality = "economy" | "balanced" | "high";

export function skinPreviewPixelRatio(quality: SkinPreviewQuality | undefined, devicePixelRatio: number): number {
    const ratio = Number.isFinite(devicePixelRatio) && devicePixelRatio > 0 ? devicePixelRatio : 1;
    if (quality === "economy") return 1;
    if (quality === "balanced") return Math.min(Math.max(ratio, 1), 1.5);
    return 2;
}
