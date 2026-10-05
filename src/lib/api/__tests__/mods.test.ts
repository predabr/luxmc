import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
const mock = vi.hoisted(() => ({ invoke: vi.fn() }));
vi.mock("../client", () => ({ api: mock }));
beforeEach(() => { vi.resetModules(); mock.invoke.mockReset(); });
afterEach(() => vi.restoreAllMocks());
describe("catalog search cache", () => {
    it("reuses recent results while keeping filters and pages separate", async () => {
        mock.invoke.mockResolvedValue([]);
        const { modsSearch } = await import("../mods");
        await modsSearch("", "1.21.1", 36, 0, "modpack");
        await modsSearch("", "1.21.1", 36, 0, "modpack");
        expect(mock.invoke).toHaveBeenCalledTimes(1);
        await modsSearch("", "1.21.1", 36, 36, "modpack");
        await modsSearch("", "1.21.1", 36, 0, "shader");
        expect(mock.invoke).toHaveBeenCalledTimes(3);
    });
    it("shares requests in flight and retries failures", async () => {
        const { modsSearch } = await import("../mods");
        let reject!: (error: Error) => void;
        mock.invoke.mockImplementationOnce(() => new Promise((_, fail) => { reject = fail; }));
        const first = modsSearch("pack", "");
        const second = modsSearch("pack", "");
        expect(mock.invoke).toHaveBeenCalledTimes(1);
        reject(new Error("offline"));
        await expect(first).rejects.toThrow("offline");
        await expect(second).rejects.toThrow("offline");
        mock.invoke.mockResolvedValue([]);
        await modsSearch("pack", "");
        expect(mock.invoke).toHaveBeenCalledTimes(2);
    });
    it("refreshes expired data", async () => {
        const time = vi.spyOn(Date, "now").mockReturnValue(1000);
        mock.invoke.mockResolvedValue([]);
        const { modsSearch } = await import("../mods");
        await modsSearch("pack", "");
        time.mockReturnValue(130000);
        await modsSearch("pack", "");
        expect(mock.invoke).toHaveBeenCalledTimes(2);
    });
});
