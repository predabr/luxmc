import {describe,it,expect} from 'vitest';
import {matchesCatalogBanner,catalogBannerCandidates,resolveProfileBanner} from './curatedBanners';

describe('catalog artwork identity',()=>{
    it('does not give a custom project another project’s curated cover',()=>{
        expect(matchesCatalogBanner('My RLCraft inspired pack','my-pack','rlcraft')).toBe(false);
        expect(matchesCatalogBanner('FO Community','community-pack','fo')).toBe(false);
        expect(matchesCatalogBanner('Society of Zombies','society-of-zombies','society')).toBe(false);
    });
    it('uses only the project provider artwork and original icon, never a name guess',()=>{const icon='https://cdn.modrinth.com/data/project/icon.png';const banner='https://cdn.modrinth.com/data/project/gallery.png';expect(catalogBannerCandidates({title:'RLCraft',slug:'rlcraft',iconUrl:icon})).toEqual([icon]);expect(catalogBannerCandidates({title:'ATM10',slug:'atm10',iconUrl:icon,bannerUrl:banner})).toEqual([banner,icon]);expect(resolveProfileBanner({name:'My RLCraft Pack',icon})).toBe(icon);});
    it('recognizes exact provider slugs across separators',()=>{
        expect(matchesCatalogBanner('Localized title','fabulously-optimized','fabulously optimized')).toBe(true);
        expect(matchesCatalogBanner('RLCraft','rlcraft','rlcraft')).toBe(true);
    });
    it('recovers the FO landscape by project identity instead of guessing a similar name',()=>{
        const icon='https://cdn.modrinth.com/data/1KVo5zza/icon.webp';
        expect(catalogBannerCandidates({title:'Renamed',slug:'renamed',source:'modrinth',sourceId:'1KVo5zza',iconUrl:icon,bannerUrl:icon})[0]).toBe('/modpack_fo.webp');
        expect(catalogBannerCandidates({title:'Fabulously Optimized',slug:'fabulously-optimized',source:'curseforge',sourceId:'396246',iconUrl:'https://media.forgecdn.net/avatars/fo.png'})[0]).toBe('/modpack_fo.webp');
        expect(catalogBannerCandidates({title:'Fabulously Optimized',slug:'fabulously-optimized',source:'modrinth',sourceId:'unrelated',iconUrl:'https://example.com/fo.png'})).toEqual(['https://example.com/fo.png']);
        expect(resolveProfileBanner({name:'Renamed FO',icon,banner:icon,loader:'fabric'})).toBe('/modpack_fo.webp');
        expect(resolveProfileBanner({name:'Renamed FO',icon,banner:'https://example.com/chosen.png'})).toBe('https://example.com/chosen.png');
    });
    it('deduplicates gallery candidates and rejects executable image URLs',()=>{
        expect(catalogBannerCandidates({title:'Pack',slug:'pack',iconUrl:'https://example.com/icon.png',bannerUrl:'https://example.com/cover.png',galleryUrls:['https://example.com/cover.png','javascript:alert(1)','https://example.com/scene.png']})).toEqual(['https://example.com/cover.png','https://example.com/scene.png','https://example.com/icon.png']);
    });
});
