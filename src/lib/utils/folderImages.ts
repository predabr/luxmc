import {prepareAnimatedImage} from './animatedImage';

const key = "luxmc_library_folder_images_v1";

export function loadFolderImages(folders: string[]): Record<string, string> {
    try {
        const saved: unknown = JSON.parse(localStorage.getItem(key) || "{}");
        if (!saved || typeof saved !== "object" || Array.isArray(saved)) return {};
        return Object.fromEntries(Object.entries(saved).filter(([name, image]) =>
            folders.includes(name) && typeof image === "string" && image.length <= 1000000 &&
            /^data:image\/(png|jpeg|webp|gif);base64,/.test(image)));
    } catch { return {}; }
}

export function saveFolderImage(images: Record<string, string>, folder: string, image: string | null): Record<string, string> {
    const next = { ...images };
    if (image === null) delete next[folder];
    else Object.defineProperty(next, folder, { value: image, enumerable: true, configurable: true, writable: true });
    localStorage.setItem(key, JSON.stringify(next));
    return next;
}

export async function prepareFolderImage(file: File): Promise<string> {
    if (!["image/png", "image/jpeg", "image/webp", "image/gif"].includes(file.type) || !file.size) throw new Error("invalid");
    if (file.size > 12 * 1024 * 1024) throw new Error("limit");
    let bitmap: ImageBitmap;
    try { bitmap = await createImageBitmap(file); } catch { throw new Error("invalid"); }
    try {
        if (file.type === "image/gif") {
            return await prepareAnimatedImage(file);
        }
        const ratio = Math.min(1, 384 / Math.max(bitmap.width, bitmap.height));
        const canvas = document.createElement("canvas");
        canvas.width = Math.max(1, Math.round(bitmap.width * ratio));
        canvas.height = Math.max(1, Math.round(bitmap.height * ratio));
        const context = canvas.getContext("2d");
        if (!context) throw new Error("invalid");
        context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
        for (const quality of [.9, .75, .6, .45]) {
            const image = canvas.toDataURL("image/webp", quality);
            if (image.length < 250000) return image;
        }
        throw new Error("limit");
    } finally { bitmap.close(); }
}
