import type { LaunchLogInput } from "$lib/api/tools";
import { sanitizeLaunchDiagnostic } from "$lib/api/launch";

export function createLaunchLogBuffer(write: (entries: LaunchLogInput[]) => void) {
    let entries: LaunchLogInput[] = [];
    let timer: ReturnType<typeof setTimeout> | undefined;
    function flush() {
        if (timer) clearTimeout(timer);
        timer = undefined;
        if (!entries.length) return;
        const batch = entries;
        entries = [];
        write(batch);
    }
    return {
        append(stream: string, level: string, message: string) {
            entries.push({ stream, level, message: sanitizeLaunchDiagnostic(message), ts: new Date().toISOString() });
            if (entries.length >= 128) flush();
            else timer ??= setTimeout(flush, 250);
        },
        flush,
    };
}
