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
});
