import { describe, it, expect } from "vitest";
import { localizedDiagnosis, type CrashDiagnosis } from "./doctor";
import { setActiveLocale } from "$lib/i18n/useTranslation.svelte";

describe("localized crash diagnoses", () => {
    const source: CrashDiagnosis = { hasError: true, title: "Sessão inválida", message: "A sessão falhou", solution: "Verifique a conta", category: "authentication", logSnippet: "Failed to log in: Invalid session", recommendedAction: "open_folder" };
    it("translates the explanation and retains the original evidence and action", () => {
        setActiveLocale("en");
        const result = localizedDiagnosis(source);
        expect(result.title).toBe("Invalid game session");
        expect(result.logSnippet).toBe(source.logSnippet);
        expect(result.recommendedAction).toBe(source.recommendedAction);
        setActiveLocale("es");
        expect(localizedDiagnosis(source).title).toBe("Sesión de juego no válida");
    });
    it("keeps the detailed native Portuguese diagnosis", () => {
        setActiveLocale("pt-BR");
        expect(localizedDiagnosis(source)).toBe(source);
    });
});
