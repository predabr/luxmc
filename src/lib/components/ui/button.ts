import { type VariantProps, tv } from "tailwind-variants";

const primary = "border border-brand-400/20 bg-brand-500 hover:bg-brand-400 active:bg-brand-600 text-brand-foreground font-extrabold shadow-sm hover:shadow-button-hover transition-all duration-150";
const secondary = "border border-fg/10 bg-fg/[0.04] hover:bg-fg/[0.08] text-fg font-bold hover:border-fg/20 hover:shadow-soft transition-all duration-150";

export const button = tv({
    base: "relative inline-flex shrink-0 items-center justify-center gap-2 overflow-hidden rounded-xl font-bold transition-all duration-150 ease-out select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-40 disabled:shadow-none aria-disabled:pointer-events-none aria-disabled:opacity-40 active:scale-[0.97] motion-reduce:transition-none",
    variants: {
        variant: {
            solid: primary,
            primary,
            secondary,
            ghost: "border-0 text-fg-muted hover:bg-fg/5 hover:text-fg active:bg-fg/10",
            outline: "border border-fg/10 bg-fg/[0.04] text-fg hover:border-fg/20 hover:bg-fg/[0.08] font-bold",
            danger: "border border-danger/30 bg-danger/10 text-danger hover:bg-danger/20 hover:border-danger/50",
            ghostBrand: "text-brand-400 hover:bg-brand-500/10 hover:text-brand-300",
            microsoft: secondary,
            play: "border border-brand-300/40 bg-gradient-to-r from-brand-600 via-brand-500 to-brand-400 text-brand-foreground font-black tracking-wider shadow-button-hover shadow-brand-500/25 hover:brightness-110"
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
