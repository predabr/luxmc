import {describe,it,expect} from 'vitest';
import {matchesCatalogBanner} from './curatedBanners';

describe('catalog artwork identity',()=>{
    it('does not give a custom project another project’s curated cover',()=>{
        expect(matchesCatalogBanner('My RLCraft inspired pack','my-pack','rlcraft')).toBe(false);
        expect(matchesCatalogBanner('FO Community','community-pack','fo')).toBe(false);
        expect(matchesCatalogBanner('Society of Zombies','society-of-zombies','society')).toBe(false);
    });
    it('recognizes exact provider slugs across separators',()=>{
        expect(matchesCatalogBanner('Localized title','fabulously-optimized','fabulously optimized')).toBe(true);
        expect(matchesCatalogBanner('RLCraft','rlcraft','rlcraft')).toBe(true);
    });
});
