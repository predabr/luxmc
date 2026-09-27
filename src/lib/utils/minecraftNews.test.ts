import { describe, expect, it } from "vitest";
import { parseMinecraftNews } from "./minecraftNews";

describe("notícias oficiais", () => {
    const entry = { id: "one", title: "Release", date: "2026-09-26", text: "Resumo", newsPageImage: { url: "/v2/images/banner.png" }, readMoreLink: "https://www.minecraft.net/article/release" };
    it("preserva datas e ordena publicações reais", () => {
        const result = parseMinecraftNews({ entries: [{ ...entry, id: "old", date: "2025-01-01" }, entry, entry] });
        expect(result).toHaveLength(2);
        expect(result[0].date).toBe("2026-09-26");
        expect(result[0].image).toBe("https://launchercontent.mojang.com/v2/images/banner.png");
    });
    it("rejeita respostas inválidas e links executáveis ou não oficiais", () => {
        expect(parseMinecraftNews(null)).toEqual([]);
        for (const readMoreLink of ["javascript:alert(1)", "https://evil.example/article", "http://minecraft.net/article"]) {
            expect(parseMinecraftNews({ entries: [{ ...entry, readMoreLink }] })).toEqual([]);
        }
        expect(parseMinecraftNews({ entries: [{ ...entry, date: "não é uma data" }] })).toEqual([]);
    });
});
