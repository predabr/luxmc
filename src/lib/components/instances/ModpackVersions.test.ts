import { beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import ModpackVersions from "./ModpackVersions.svelte";
import { setActiveLocale } from "$lib/i18n/useTranslation.svelte";

const mock = vi.hoisted(() => ({ versions: vi.fn(), apply: vi.fn(), refresh: vi.fn(), toast: vi.fn(), game: { isGameRunning: false, isLaunching: false } }));
vi.mock("$lib/api/mods", () => ({ modsVersions: mock.versions }));
vi.mock("$lib/api/java", () => ({ modpackUpdateAtomic: mock.apply }));
vi.mock("$lib/stores/app.svelte", () => ({ appState: mock.game }));
vi.mock("$lib/stores/profiles.svelte", () => ({ profiles: { refresh: mock.refresh } }));
vi.mock("$lib/stores/toasts.svelte", () => ({ toast: mock.toast }));

beforeEach(() => {
    setActiveLocale("pt-BR");
    cleanup(); vi.clearAllMocks(); mock.game.isGameRunning = false;
    mock.versions.mockResolvedValue([{ id: "selected-version", name: "Author release", versionNumber: "2.0", files: [] }]);
    mock.apply.mockResolvedValue(undefined); mock.refresh.mockResolvedValue(undefined);
});

async function open() {
    const changed = vi.fn().mockResolvedValue(undefined);
    const view = render(ModpackVersions, { profileId: "profile", info: { projectId: "pack", source: "modrinth", currentVersion: "1.0", hasUpdate: false, latestVersion: null, changelog: null, versionId: null }, onChanged: changed });
    const details = view.container.querySelector("details")!;
    details.open = true; await fireEvent(details, new Event("toggle"));
    await screen.findByRole("option", { name: "2.0 · Author release" });
    return changed;
}

describe("modpack version replacement", () => {
    it("requires explicit confirmation and refreshes only after a successful replacement", async () => {
        const changed = await open();
        await fireEvent.click(screen.getByRole("button", { name: "Escolher versão" }));
        expect(mock.apply).not.toHaveBeenCalled();
        await fireEvent.click(screen.getByRole("button", { name: "Fazer backup e aplicar" }));
        await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
        expect(mock.apply).toHaveBeenCalledWith("profile", "selected-version", "modrinth", "pack");
        expect(mock.refresh).toHaveBeenCalledOnce();
    });
    it("blocks replacement while Minecraft is running", async () => {
        mock.game.isGameRunning = true; await open();
        expect((screen.getByRole("button", { name: "Escolher versão" }) as HTMLButtonElement).disabled).toBe(true);
        expect(mock.apply).not.toHaveBeenCalled();
    });
});
