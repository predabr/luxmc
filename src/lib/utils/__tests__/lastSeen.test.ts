import { afterEach, describe, expect, it } from "vitest";
import { setActiveLocale } from "$lib/i18n/useTranslation.svelte";
import { lastSeenLabel } from "../lastSeen";

afterEach(() => setActiveLocale("pt-BR"));

describe("localized friend activity", () => {
    it("uses each language for singular and plural day and hour labels", () => {
        const now = Date.parse("2026-10-06T12:00:00Z");
        setActiveLocale("en");
        expect(lastSeenLabel("2026-10-05T12:00:00Z", now)).toBe("Seen 1 day ago");
        expect(lastSeenLabel("2026-10-06T10:00:00Z", now)).toBe("Seen 2 hours ago");
        setActiveLocale("es");
        expect(lastSeenLabel("2026-10-05T12:00:00Z", now)).toBe("Visto hace 1 día");
        setActiveLocale("pt-BR");
        expect(lastSeenLabel("2026-10-04T12:00:00Z", now)).toBe("Visto há 2 dias");
    });
});
