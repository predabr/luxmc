import { describe, expect, it } from "vitest";
import { parseDeepLink } from "../deepLink";

describe("portal protocol", () => {
    it("opens projects from either provider", () => {
        expect(parseDeepLink("luxmc://install/mod?id=sodium&source=modrinth")).toEqual({ kind: "install", content: "mod", id: "sodium", source: "modrinth" });
        expect(parseDeepLink("luxmc://install/modpack?id=396246&source=curseforge")).toEqual({ kind: "install", content: "modpack", id: "396246", source: "curseforge" });
    });
    it("preserves encoded skin URLs and model", () => {
        const url = "https://example.org/skin.png?version=2&name=Alex";
        expect(parseDeepLink(`luxmc://skin/apply?${new URLSearchParams({ url, model: "slim" })}`)).toEqual({ kind: "skin", url, model: "slim" });
    });
    it("accepts modern and legacy server links and IPv6", () => {
        expect(parseDeepLink("luxmc://join/server?ip=play.example.org&port=25566")).toEqual({ kind: "server", address: "play.example.org:25566" });
        expect(parseDeepLink("luxmc://join/play.example.org:25565")).toEqual({ kind: "server", address: "play.example.org:25565" });
        expect(parseDeepLink("luxmc://join/server?ip=2001:db8::1")).toEqual({ kind: "server", address: "[2001:db8::1]:25565" });
        expect(parseDeepLink("luxmc://join/friend?code=friend-id")).toEqual({ kind: "friend", code: "friend-id" });
    });
    it.each([
        "https://install/mod?id=sodium&source=modrinth",
        "luxmc://install/mod?id=a&source=unknown",
        "luxmc://install/mod?id=../escape&source=modrinth",
        "luxmc://install/mod?id=a&source=curseforge",
        "luxmc://install/mod?id=a&id=b&source=modrinth",
        "luxmc://skin/apply?url=javascript:alert(1)&model=classic",
        "luxmc://skin/apply?url=file:///tmp/skin.png&model=classic",
        "luxmc://skin/apply?url=https://user:secret@example.org/a&model=slim",
        "luxmc://skin/apply?url=https://example.org/a&model=unknown",
        "luxmc://join/server?ip=host&port=65536",
        "luxmc://join/server?ip=host&port=0",
        "luxmc://join/server?ip=host%20--jvm-flag",
        "luxmc://join/server?ip=user@host",
        "luxmc://join/friend?code=",
        "luxmc://user@install/mod?id=a&source=modrinth",
        "luxmc://install/mod?id=a&source=modrinth#fragment",
        "luxmc://delete/instance?id=all"
    ])("rejects malformed or unsupported actions: %s", value => {
        expect(() => parseDeepLink(value)).toThrow();
    });
});
