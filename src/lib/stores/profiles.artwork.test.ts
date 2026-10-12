import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { profiles, type Profile } from "./profiles.svelte";

const mocks = vi.hoisted(() => ({ search: vi.fn(), update: vi.fn(), reference: vi.fn() }));
vi.mock("$lib/api/mods", () => ({ modsSearch: mocks.search }));
vi.mock("$lib/api/instances", () => ({ profilesUpdate: mocks.update }));
vi.mock("$lib/api/studio", () => ({ instancePackReference: mocks.reference }));

const artwork = { title: "Vulkan Optimized", iconUrl: "https://cdn.modrinth.com/official-pack.webp" };
const profile = (id: string, icon = "grass_block", loader: Profile["loader"] = "fabric"): Profile => ({
 id, name: "Vulkan Optimized", icon, loader, mcVersion: "26.1.2", gameDir: "C:/Luxmc/test", createdAt: 1, updatedAt: 1
});

beforeEach(() => { mocks.search.mockReset(); mocks.update.mockReset(); mocks.update.mockResolvedValue(undefined); mocks.reference.mockReset(); mocks.reference.mockResolvedValue(null); localStorage.clear(); });
afterEach(() => { profiles.list = []; vi.restoreAllMocks(); });

describe("profile artwork recovery", () => {
 it("recovers and persists an installed FO cover using its CurseForge reference", async () => {
  mocks.reference.mockResolvedValue({source:"curseforge",projectId:"396246",versionId:"test",name:"Fabulously Optimized"});
  profiles.list = [profile("fo-cover", "https://media.forgecdn.net/avatars/official-fo.png")];
  await vi.waitFor(() => expect(profiles.list[0].banner).toBe("/modpack_fo.webp"));
  expect(localStorage.getItem("luxmc_catalog_banner_fo-cover")).toBe("/modpack_fo.webp");
  profiles.list = [profile("fo-cover", "https://media.forgecdn.net/avatars/official-fo.png")];
  expect(profiles.list[0].banner).toBe("/modpack_fo.webp");
 });

 it("preserves a custom cover chosen while recovery is pending", async () => {
  let resolve!: (value: unknown) => void;
  mocks.reference.mockReturnValue(new Promise(done => {resolve=done;}));
  profiles.list = [profile("cover-pending", "https://example.com/pack.png")];
  await vi.waitFor(()=>expect(mocks.reference).toHaveBeenCalledTimes(1));
  profiles.setBanner("cover-pending","https://example.com/chosen.png");
  resolve({source:"curseforge",projectId:"396246",versionId:"test"});
  await new Promise(done=>setTimeout(done,30));
  expect(profiles.list[0].banner).toBe("https://example.com/chosen.png");
  expect(localStorage.getItem("luxmc_catalog_banner_cover-pending")).toBeNull();
 });
 it("persists an exact catalog match and preserves Vanilla and custom icons", async () => {
  mocks.search.mockResolvedValue([artwork]);
  profiles.list = [profile("artwork-exact"), profile("artwork-vanilla", "grass_block", "vanilla"), profile("artwork-custom", "/custom.png")];
  await vi.waitFor(() => expect(profiles.list[0].icon).toBe(artwork.iconUrl));
  expect(mocks.update).toHaveBeenCalledExactlyOnceWith({id:"artwork-exact",icon:artwork.iconUrl});
  expect(mocks.search).toHaveBeenCalledTimes(1);
  expect(profiles.list[1].icon).toBe("grass_block");
  expect(profiles.list[2].icon).toBe("/custom.png");
 });

 it("does not overwrite an icon selected while catalog lookup was pending", async () => {
  let resolve!: (items: typeof artwork[]) => void;
  mocks.search.mockReturnValue(new Promise<typeof artwork[]>(done => { resolve = done; }));
  profiles.list = [profile("artwork-pending")];
  await vi.waitFor(() => expect(mocks.search).toHaveBeenCalledTimes(1));
  profiles.update("artwork-pending", {icon:"/chosen.png"});
  resolve([artwork]);
  await new Promise(done => setTimeout(done, 30));
  expect(profiles.list[0].icon).toBe("/chosen.png");
  expect(mocks.update).not.toHaveBeenCalled();
 });

 it("retries a failed lookup after backoff instead of blocking recovery for the session", async () => {
  let now = 1000;
  vi.spyOn(Date, "now").mockImplementation(() => now);
  mocks.search.mockRejectedValueOnce(new Error("offline")).mockResolvedValue([artwork]);
  profiles.list = [profile("artwork-retry")];
  await vi.waitFor(() => expect(mocks.search).toHaveBeenCalledTimes(1));
  profiles.list = [profile("artwork-retry")];
  await new Promise(done => setTimeout(done, 30));
  expect(mocks.search).toHaveBeenCalledTimes(1);
  now += 300001;
  profiles.list = [profile("artwork-retry")];
  await vi.waitFor(() => expect(profiles.list[0].icon).toBe(artwork.iconUrl));
  expect(mocks.search).toHaveBeenCalledTimes(2);
 });
});
