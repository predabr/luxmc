export function mergeSavedSettings<T extends { settingsSavedAt?: number }>(native: Partial<T> | null, backup: Partial<T> | null): Partial<T> {
    const primary: Partial<T> = native ?? {};
    const secondary: Partial<T> = backup ?? {};
    const nativeTime = primary.settingsSavedAt ?? 0;
    const backupTime = secondary.settingsSavedAt ?? 0;
    return backupTime > nativeTime || (!nativeTime && !backupTime)
        ? { ...primary, ...secondary }
        : { ...secondary, ...primary };
}
