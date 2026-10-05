import { describe, expect, it } from "vitest";
import { CapeObject } from "skinview3d";
import { createCapeCloth } from "./capeCloth";
describe("cape cloth", () => {
    it("keeps the neck anchored while moving the hem and restores the resting shape", () => {
        const mesh = new CapeObject().cape;
        const cloth = createCapeCloth(mesh);
        const rest = Array.from(mesh.geometry.getAttribute("position").array);
        const uv = Array.from(mesh.geometry.getAttribute("uv").array);
        expect(uv.every(Number.isFinite)).toBe(true);
        cloth.update(1, 1, .2, .8, true);
        const moving = Array.from(mesh.geometry.getAttribute("position").array);
        expect(moving.every(Number.isFinite)).toBe(true);
        expect(moving).not.toEqual(rest);
        for (let i = 0; i < rest.length; i += 3) {
            if (rest[i + 1] === 8) expect(moving.slice(i, i + 3)).toEqual(rest.slice(i, i + 3));
        }
        cloth.update(2, 1, .5, 1, false);
        expect(Array.from(mesh.geometry.getAttribute("position").array)).toEqual(rest);
        expect(createCapeCloth(mesh)).toBe(cloth);
        mesh.geometry.dispose();
    });
});
