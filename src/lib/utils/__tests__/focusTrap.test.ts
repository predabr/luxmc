import { afterEach, describe, expect, it, vi } from "vitest";
import { focusTrap } from "../focusTrap";

afterEach(() => { document.body.replaceChildren(); vi.restoreAllMocks(); });

function dialog() {
    const element = document.createElement("div");
    element.tabIndex = -1;
    element.innerHTML = '<button>Primeiro</button><button disabled>Indisponível</button><button>Último</button>';
    document.body.append(element);
    for (const control of element.querySelectorAll("button")) vi.spyOn(control, "getClientRects").mockReturnValue(Object.assign([new DOMRect()], { item: () => new DOMRect() }));
    return element;
}

describe("modal keyboard focus", () => {
    it("wraps focus, excludes disabled actions and restores the opener", async () => {
        const opener = document.createElement("button");
        document.body.append(opener);
        opener.focus();
        const element = dialog();
        const trap = focusTrap(element);
        await Promise.resolve();
        expect(document.activeElement).toBe(element.firstElementChild);
        document.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", shiftKey: true, cancelable: true }));
        expect(document.activeElement).toBe(element.lastElementChild);
        document.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", cancelable: true }));
        expect(document.activeElement).toBe(element.firstElementChild);
        opener.focus();
        expect(document.activeElement).toBe(element.firstElementChild);
        trap.destroy();
        expect(document.activeElement).toBe(opener);
    });

    it("lets the top dialog own focus without trapping the nested modal outside", async () => {
        const parent = dialog();
        const first = focusTrap(parent);
        await Promise.resolve();
        const child = dialog();
        const second = focusTrap(child);
        await Promise.resolve();
        expect(document.activeElement).toBe(child.firstElementChild);
        second.destroy();
        child.remove();
        expect(document.activeElement).toBe(parent.firstElementChild);
        first.destroy();
    });
});
