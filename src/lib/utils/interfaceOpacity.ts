export function interfaceOpacityRatio(value?: number): number {
    if (value === undefined || !Number.isFinite(value)) return .18;
    return Math.max(.05, Math.min(.9, value <= 1 ? value : value / 100));
}
