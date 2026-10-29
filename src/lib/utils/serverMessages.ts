export function serverMessage(cause: unknown): string {
    return (cause instanceof Error ? cause.message : String(cause)).replace(/^(?:(?:Error|invalid state|invalid input|serialization error):\s*)+/i, '');
}
