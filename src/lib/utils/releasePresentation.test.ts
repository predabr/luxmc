import { describe,it,expect } from 'vitest';
import { releaseImages,releaseSections } from './releasePresentation';
describe('release presentation', () => {
    it('only displays images from approved HTTPS origins', () => {
        expect(releaseImages('![x](https://evil.test/a.png) ![x](https://user-images.githubusercontent.com/a.png) ![x](https://user-images.githubusercontent.com/a.png)')).toEqual(['https://user-images.githubusercontent.com/a.png']);
    });
    it('groups highlights without treating text as HTML', () => {
        expect(releaseSections('## Downloads\n- **Resume** safely\n## Worlds\n- Backups')).toEqual([{title:'Downloads',items:['Resume safely']},{title:'Worlds',items:['Backups']}]);
    });
});
