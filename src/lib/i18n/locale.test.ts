import { describe, expect, it } from "vitest";
import { resolveLocale } from "./locale";

describe("system language mapping", () => {
	it.each([
		["pt-PT", "pt-BR"], ["pt-BR", "pt-BR"], ["pt-AO", "pt-BR"], ["PT_mz", "pt-BR"],
		["pt", "pt-BR"], ["es-MX", "es"], ["es-ES", "es"], ["es", "es"],
		["en-GB", "en"], ["de-DE", "en"], ["fr-FR", "en"], ["ja-JP", "en"], ["", "en"],
		[undefined, "en"], ["ptinvalid", "en"],
	])("maps %s to %s", (source, expected) => expect(resolveLocale(source)).toBe(expected));
});
