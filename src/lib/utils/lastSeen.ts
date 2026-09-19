export function lastSeenLabel(value: string | null | undefined, now: number = Date.now()): string {
    if (!value) return "Offline";
    const timestamp = Date.parse(value);
    if (!Number.isFinite(timestamp)) return "Offline";
    const minutes = Math.max(0, Math.floor((now - timestamp) / 60000));
    if (minutes < 1) return "Visto agora há pouco";
    if (minutes < 60) return `Visto há ${minutes} min`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `Visto há ${hours} ${hours === 1 ? "hora" : "horas"}`;
    const days = Math.floor(hours / 24);
    return `Visto há ${days} ${days === 1 ? "dia" : "dias"}`;
}
