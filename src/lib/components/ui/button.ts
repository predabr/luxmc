import { type VariantProps, tv } from "tailwind-variants";

const primary = "border-0 bg-[#1bd96a] hover:bg-[#18c45f] active:bg-[#15af54] text-[#090a0f] font-extrabold shadow-sm transition-all duration-150 active:scale-[0.98]";
const secondary = "border border-fg/10 bg-bg-elevated hover:bg-bg-subtle text-fg font-bold hover:border-fg/20 transition-all duration-150 active:scale-[0.98]";

export const button = tv({
    base: "relative inline-flex shrink-0 items-center justify-center gap-2 overflow-hidden rounded-xl font-bold transition-all duration-150 ease-out select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#1bd96a] focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-40 disabled:shadow-none aria-disabled:pointer-events-none aria-disabled:opacity-40 active:scale-[0.98]",
    variants: {
        variant: {
            solid: primary,
            primary,
            secondary,
            ghost: "border-0 text-fg-muted hover:bg-fg/5 hover:text-fg active:bg-fg/10",
            outline: "border border-[#1bd96a]/40 bg-[#1bd96a]/10 text-[#1bd96a] hover:border-[#1bd96a]/70 hover:bg-[#1bd96a]/20 font-bold",
            danger: "border border-danger/30 bg-danger/10 text-danger hover:bg-danger/20 hover:border-danger/50",
            ghostBrand: "text-[#1bd96a] hover:bg-[#1bd96a]/10 hover:text-[#18c45f]",
            microsoft: secondary,
            play: "border-0 bg-[#1bd96a] hover:bg-[#18c45f] active:bg-[#15af54] text-[#090a0f] font-black tracking-wide shadow-sm transition-all duration-150 active:scale-[0.98]"
        },
        size: {
            sm: "h-9 px-3 text-xs rounded-xl",
            md: "h-10 px-4 text-xs rounded-xl",
            lg: "h-11 px-5 text-sm rounded-xl",
            xl: "h-12 px-6 text-sm rounded-2xl",
            pill: "h-9 px-5 text-xs rounded-full",
            hero: "h-14 px-8 text-sm tracking-wide rounded-2xl",
            icon: "h-10 w-10 p-0 rounded-xl"
        },
        block: { true: "w-full" }
    },
    defaultVariants: { variant: "solid", size: "md" }
});
export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
