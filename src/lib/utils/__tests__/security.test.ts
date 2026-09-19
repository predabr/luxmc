import { describe, expect, it } from "vitest";
import { parseJoinAddress } from "../joinAddress";
import { sanitizeHtml } from "../sanitizeHtml";

describe("addresses from friends and deep links", () => {
    it("parses direct addresses and bracketed IPv6", () => {
        expect(parseJoinAddress("luxmc://join/example.org:25566")).toEqual({ host: "example.org", port: 25566 });
        expect(parseJoinAddress("[2001:db8::1]:25565")).toEqual({ host: "[2001:db8::1]", port: 25565 });
    });
    it.each(["LUX-1234", "host:0", "host:65536", "host/path", "user@host", "host --flag", "https://host"])('rejects unsafe or unresolvable input %s', value => {
        expect(() => parseJoinAddress(value)).toThrow();
    });
});

describe("untrusted mod descriptions", () => {
    it("removes scripts, event handlers, frames and executable links", () => {
        const result = sanitizeHtml('<script>alert(1)</script><img src="image.png" onerror="alert(2)"><a href="javascript:alert(3)">link</a><iframe srcdoc="bad"></iframe><form><input></form>');
        expect(result).not.toMatch(/script|onerror|iframe|srcdoc|javascript:|<form|<input/i);
        expect(result).toContain('src="image.png"');
    });
    it("keeps normal formatted documentation and links", () => {
        expect(sanitizeHtml('<h2>Guide</h2><p><strong>Install</strong> <a href="https://modrinth.com">here</a></p>')).toContain('<strong>Install</strong>');
    });
});
