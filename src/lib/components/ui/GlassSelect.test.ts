import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@testing-library/svelte";
import GlassSelect from "./GlassSelect.svelte";

beforeEach(() => {
	cleanup();
	window.HTMLElement.prototype.hidePopover = () => {};
	window.HTMLElement.prototype.showPopover = () => {};
});

afterEach(() => {
	vi.restoreAllMocks();
	cleanup();
});

describe("GlassSelect window listeners", () => {
	it("survives a window resize event whose target is the window, not a node", async () => {
		const view = render(GlassSelect, {
			label: "Fonte",
			value: "modrinth",
			options: [
				{ value: "modrinth", label: "Modrinth" },
				{ value: "curseforge", label: "CurseForge" }
			]
		});

		const trigger = view.container.querySelector("button");
		expect(trigger).toBeTruthy();
		await fireEvent.click(trigger!);

		const errors: unknown[] = [];
		const collect = (event: ErrorEvent) => errors.push(event.error ?? event.message);
		window.addEventListener("error", collect);
		window.dispatchEvent(new Event("resize"));
		window.dispatchEvent(new Event("scroll"));
		await Promise.resolve();
		window.removeEventListener("error", collect);
		expect(errors).toEqual([]);
	});

	it("keeps the popup open when the resize target is inside it", async () => {
		const view = render(GlassSelect, {
			label: "Fonte",
			value: "modrinth",
			options: [{ value: "modrinth", label: "Modrinth" }]
		});

		await fireEvent.click(view.container.querySelector("button")!);
		const popup = document.body.querySelector<HTMLElement>('[role="listbox"]');
		expect(popup).toBeTruthy();

		const inner = document.createElement("span");
		popup!.appendChild(inner);
		expect(() => inner.dispatchEvent(new Event("resize", { bubbles: true }))).not.toThrow();
	});
});
