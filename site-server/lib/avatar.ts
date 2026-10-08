export function validAvatar(value: unknown) {
    if (typeof value !== "string" || value.length > 2048)
        return false;
    if (value.startsWith("data:image/png;base64,")) {
        try {
            const bytes = Uint8Array.from(atob(value.slice(22)), (character: any) => character.charCodeAt(0));
            if (bytes.length < 24 || [137, 80, 78, 71, 13, 10, 26, 10].some((byte: any, index: any) => bytes[index] !== byte))
                return false;
            const view = new DataView(bytes.buffer);
            return [view.getUint32(16), view.getUint32(20)].every((size: any) => size > 0 && size <= 64);
        }
        catch {
            return false;
        }
    }
    try {
        const url = new URL(value);
        return url.protocol === "https:" && Boolean(url.hostname) && !url.username && !url.password;
    }
    catch {
        return false;
    }
}
