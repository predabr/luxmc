import { afterEach, beforeEach, expect, it, vi } from "vitest";
vi.mock("$lib/api/launch", () => ({ sanitizeLaunchDiagnostic: (value: string) => value.replace(/Bearer \S+/g, "[redacted]") }));
import { createLaunchLogBuffer } from "./launchLogBuffer";

beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(new Date("2026-10-13T12:00:00Z")); });
afterEach(() => vi.useRealTimers());

it("writes bursts in bounded batches while preserving every line and its timestamp", () => {
    const write = vi.fn();
    const buffer = createLaunchLogBuffer(write);
    for (let i = 0; i < 300; i++) buffer.append("stdout", "INFO", String(i));
    expect(write.mock.calls.map(([batch]) => batch.length)).toEqual([128, 128]);
    vi.advanceTimersByTime(250);
    expect(write.mock.calls.map(([batch]) => batch.length)).toEqual([128, 128, 44]);
    const lines = write.mock.calls.flatMap(([batch]) => batch);
    expect(lines.map(line => line.message)).toEqual(Array.from({ length: 300 }, (_, i) => String(i)));
    expect(lines.every(line => line.ts === "2026-10-13T12:00:00.000Z")).toBe(true);
});

it("flushes the remaining diagnostics before closing and cancels its timer", () => {
    const write = vi.fn();
    const buffer = createLaunchLogBuffer(write);
    buffer.append("stderr", "WARN", "Bearer secret-token");
    buffer.flush();
    expect(write).toHaveBeenCalledExactlyOnceWith([{ stream: "stderr", level: "WARN", message: "[redacted]", ts: "2026-10-13T12:00:00.000Z" }]);
    vi.advanceTimersByTime(1000);
    expect(write).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
});
