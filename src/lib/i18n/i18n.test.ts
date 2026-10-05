import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import pt from "./pt-BR.json";
import en from "./en.json";
import es from "./es.json";
function flatten(value: Record<string, unknown>, prefix = ""): Record<string, string> {
    return Object.fromEntries(Object.entries(value).flatMap(([key, item]) => {
        const path = prefix ? `${prefix}.${key}` : key;
        return typeof item === "string" ? [[path, item]] : Object.entries(flatten(item as Record<string, unknown>, path));
    }));
}
function files(directory: string): string[] {
    return readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(join(directory, entry.name)) : /\.(svelte|ts)$/.test(entry.name) ? [join(directory, entry.name)] : []);
}
const dictionaries = { en, "pt-BR": pt, es } as const;
function missingKeys(source: Record<string, string>, target: Record<string, string>): string[] {
    return Object.keys(source).filter(key => !target[key]);
}
describe("Português do Brasil", () => {
    it("cobre todas as chaves dos outros idiomas e mantém os parâmetros", () => {
        const translated = flatten(pt);
        for (const dictionary of [en, es]) for (const [key, value] of Object.entries(flatten(dictionary))) {
            expect(translated[key], key).toBeTruthy();
            expect((translated[key].match(/{{[^}]+}}/g) || []).sort(), key).toEqual((value.match(/{{[^}]+}}/g) || []).sort());
        }
    });
});
describe("Cobertura dos idiomas", () => {
    it("cobre todas as chaves do inglês e mantém os parâmetros", () => {
        const sources = Object.fromEntries(Object.entries(dictionaries).map(([locale, dictionary]) => [locale, flatten(dictionary)]));
        for (const [locale, translated] of Object.entries(sources)) {
            expect(missingKeys(sources.en, translated), locale).toEqual([]);
            expect(missingKeys(sources["pt-BR"], translated), locale).toEqual([]);
            for (const [key, value] of Object.entries(sources.en)) {
                expect((translated[key].match(/{{[^}]+}}/g) || []).sort(), `${locale}: ${key}`).toEqual((value.match(/{{[^}]+}}/g) || []).sort());
            }
        }
    });
    it("resolve as chaves literais usadas nas telas em todos os idiomas", () => {
        const translated = Object.fromEntries(Object.entries(dictionaries).map(([locale, dictionary]) => [locale, flatten(dictionary)]));
        for (const file of files("src")) {
            const source = readFileSync(file, "utf8");
            for (const match of source.matchAll(/\b(?:t|uiText)\(["']([\w.-]+)["']/g)) {
                for (const [locale, dictionary] of Object.entries(translated)) expect(dictionary[match[1]], `${locale} ${file}: ${match[1]}`).toBeTruthy();
            }
        }
    });
});
