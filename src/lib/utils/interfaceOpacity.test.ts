import {describe,it,expect} from 'vitest';
import {interfaceOpacityRatio} from './interfaceOpacity';

describe('wallpaper opacity across saved settings and synchronized profiles', () => {
    it.each([[18,.18],[35,.35],[90,.9],[.18,.18],[.35,.35],[1,.9],[0,.05],[NaN,.18],[undefined,.18]])('normalizes %s to %s',(value,expected) => {
        expect(interfaceOpacityRatio(value)).toBe(expected);
    });
});
