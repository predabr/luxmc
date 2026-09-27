import { describe, expect, it } from "vitest";
import { isSkinTexture, isCapeTexture, isCapeUVPattern, inferSkinModelType, classifyTexture } from "./textureImage";

function texture(width: number, height: number, fill: (x: number, y: number) => boolean): HTMLCanvasElement {
    const canvas = { width, height } as HTMLCanvasElement;
    const context = { canvas, getImageData(x: number, y: number, w: number, h: number) {
        const data = new Uint8ClampedArray(w * h * 4);
        for (let row = 0; row < h; row++) for (let col = 0; col < w; col++) {
            const index = (row * w + col) * 4;
            data[index] = 80;
            data[index + 3] = fill(x + col, y + row) ? 255 : 0;
        }
        return { data };
    } } as CanvasRenderingContext2D;
    canvas.getContext = (() => context) as unknown as typeof canvas.getContext;
    return canvas;
}

describe("identificação de texturas", () => {
    it("aceita skins modernas, legadas e HD sem aceitar miniaturas", () => {
        for (const [width, height] of [[64,64],[64,32],[128,128],[512,256]]) expect(isSkinTexture(width,height)).toBe(true);
        for (const [width, height] of [[8,8],[22,17],[65,65],[4096,4096]]) expect(isSkinTexture(width,height)).toBe(false);
    });
    it("distingue o UV de capa de uma skin legada com as mesmas dimensões", () => {
        const cape = texture(64,32,(x,y) => x < 22 && y < 17);
        const skin = texture(64,32,() => true);
        expect(isCapeTexture(64,32)).toBe(true);
        expect(isCapeUVPattern(cape.getContext("2d")!)).toBe(true);
        expect(classifyTexture(cape)).toBe("cape");
        expect(classifyTexture(skin)).toBe("skin");
        expect(inferSkinModelType(skin)).toBe("steve");
    });
    it("reconhece OptiFine proporcional e nomes de capas opacas", () => {
        expect(classifyTexture(texture(44,34,() => true))).toBe("cape");
        expect(classifyTexture(texture(64,32,() => true), "cape Pink Cherry Blossom.png")).toBe("cape");
        expect(() => classifyTexture(texture(80,20,() => true))).toThrow();
    });
    it("não classifica uma skin preta como capa", () => {
        expect(classifyTexture(texture(64,64,() => true))).toBe("skin");
        expect(inferSkinModelType(texture(64,64,() => true))).toBe("steve");
    });
    it("detecta os braços finos e preserva modelo clássico legado", () => {
        const slim = texture(64,64,(x,y) => !(x >= 54 && x < 56 && y >= 20 && y < 32));
        expect(inferSkinModelType(slim)).toBe("alex");
        expect(inferSkinModelType(texture(64,32,() => false))).toBe("steve");
    });
});
