import { describe, expect, it, vi } from "vitest";
import type { ModProjectDetails } from "$lib/api/types";
import { catalogGallery, latestMinecraftVersion, descriptionArtwork } from "./catalogGallery";

const mocks = vi.hoisted(() => ({ details: vi.fn() }));
vi.mock("$lib/api/mods", () => ({ modsProjectDetails: mocks.details }));
const details = (id: string): ModProjectDetails => ({ id, source:"modrinth", slug:id, title:id, description:"", body:"", bodyType:"markdown", downloads:0, iconUrl:null, categories:[], loaders:[], gameVersions:[], gallery:[{url:`https://cdn.modrinth.com/data/${id}/landscape.png`}] });

describe("catalog gallery recovery", () => {
    it('recovers author cover images from descriptions without sponsor cards or unsafe URLs', () => {
        expect(descriptionArtwork('<img src="https://bisecthosting.com/images/CF/RLCraft/BH_RL_Header.webp"><img src="https://example.com/PromoCard.png"><img src="javascript:alert(1)"> ![Scene](https://cdn.modrinth.com/scene.png)')).toEqual(['https://bisecthosting.com/images/CF/RLCraft/BH_RL_Header.webp', 'https://cdn.modrinth.com/scene.png']);
    });
    it('keeps the full resolution project icon available when no gallery exists', async () => {
        mocks.details.mockResolvedValueOnce({ ...details('original-icon'), gallery: [], iconUrl: 'https://cdn.modrinth.com/data/original-icon/original.png' });
        expect(await catalogGallery('modrinth', 'original-icon')).toEqual(['https://cdn.modrinth.com/data/original-icon/original.png']);
    });
	it("deduplicates pending requests and caches the gallery of the same project", async () => {
		mocks.details.mockResolvedValueOnce(details("cache-pack"));
		const results = await Promise.all([catalogGallery("modrinth","cache-pack"),catalogGallery("modrinth","cache-pack")]);
		expect(results[0]).toEqual(["https://cdn.modrinth.com/data/cache-pack/landscape.png"]);
		expect(await catalogGallery("modrinth","cache-pack")).toEqual(results[0]);
		expect(mocks.details).toHaveBeenCalledTimes(1);
	});
	it("does not use a fallback response belonging to another project", async () => {
		mocks.details.mockResolvedValueOnce(details("another-pack"));
		expect(await catalogGallery("modrinth","requested-pack")).toEqual([]);
	});
	it("limits gallery requests to three at a time", async () => {
		let running=0;
		let maximum=0;
		mocks.details.mockImplementation(async (id:string) => {
			running+=1;
			maximum=Math.max(maximum,running);
			await new Promise(resolve=>setTimeout(resolve,10));
			running-=1;
			return details(id);
		});
		await Promise.all(Array.from({length:9},(_,index)=>catalogGallery("modrinth",`bounded-${index}`)));
		expect(maximum).toBe(3);
	});
	it("keeps a failed gallery lookup from interrupting the catalog", async () => {
		mocks.details.mockRejectedValueOnce(new Error("offline"));
		expect(await catalogGallery("modrinth","offline-pack")).toEqual([]);
	});
	it("never displays loader names as Minecraft versions", () => {
		expect(latestMinecraftVersion(["Forge","Fabric","Java 17","1.20.1","1.12.2"])).toBe("1.20.1");
		expect(latestMinecraftVersion(["NeoForge","26.3","26.2"])).toBe("26.3");
		expect(latestMinecraftVersion(["Forge"])).toBeUndefined();
		expect(latestMinecraftVersion(["24w40a"])).toBe("24w40a");
	});
});
