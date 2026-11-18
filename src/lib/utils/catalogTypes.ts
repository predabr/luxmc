import { Box, Cpu, Palette, Sparkles, HardDrive, Globe } from "lucide-svelte";

export const catalogTypes = [
	{ id: "Modpack", labelKey: "ui.ff74891e9da6c4d7", icon: Box },
	{ id: "Mod", labelKey: "ui.2fe83d132e3db243", icon: Cpu },
	{ id: "Resource Pack", labelKey: "ui.7ca53deac08c5017", icon: Palette },
	{ id: "Shader", labelKey: "ui.c2385aa66c95026c", icon: Sparkles },
	{ id: "Data Pack", labelKey: "ui.37db17c75522082f", icon: HardDrive },
	{ id: "World", labelKey: "ui.78ae647dc5544d22", icon: Globe }
] as const;

export type CatalogTypeId = (typeof catalogTypes)[number]["id"];
