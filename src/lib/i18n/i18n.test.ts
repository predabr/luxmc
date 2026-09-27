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
describe("Português do Brasil", () => {
    it("cobre todas as chaves dos outros idiomas e mantém os parâmetros", () => {
        const translated = flatten(pt);
        for (const dictionary of [en, es]) for (const [key, value] of Object.entries(flatten(dictionary))) {
            expect(translated[key], key).toBeTruthy();
            expect((translated[key].match(/{{[^}]+}}/g) || []).sort(), key).toEqual((value.match(/{{[^}]+}}/g) || []).sort());
        }
    });
    it("resolve as chaves literais usadas nas telas", () => {
        const translated = flatten(pt);
        for (const file of files("src")) {
            const source = readFileSync(file, "utf8");
            for (const match of source.matchAll(/\bt\(["']([\w.-]+)["']/g)) expect(translated[match[1]], `${file}: ${match[1]}`).toBeTruthy();
        }
    });
});
