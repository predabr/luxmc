import { expect, it } from 'vitest';
import { distinctFavourites, favouriteArtwork, safeFavouriteIcon } from './profileFavourites';
it('shows one canonical name and the stored cover without requiring a local installation', () => {
    const artwork = [{name:'pixelmon',title:'The Pixelmon Modpack',icon:'https://media.forgecdn.net/avatars/pixelmon.png'}];
    expect(distinctFavourites(['pixelmon','The Pixelmon Modpack'],artwork)).toEqual(artwork);
    expect(favouriteArtwork('pixelmon',artwork).icon).toBe(artwork[0].icon);
});
it('uses library covers for typed aliases and keeps unknown projects with a fallback', () => {
    const library = [{name:'The Pixelmon Modpack',icon:'https://cdn.modrinth.com/data/pack/icon.png'}];
    expect(favouriteArtwork('pixelmon',[],library).title).toBe('The Pixelmon Modpack');
    expect(favouriteArtwork('unlisted',[],library).icon).toBe('/grass_block.png');
    for (const icon of ['file:///C:/private.png','https://cdn.modrinth.com.evil.test/logo.png','https://user@cdn.modrinth.com/logo.png','data:image/svg+xml,x']) expect(safeFavouriteIcon(icon)).toBe(false);
});
