import { type VariantProps, tv } from "tailwind-variants";

const primary = "border border-brand-400/40 bg-gradient-to-r from-brand-600 to-indigo-600 text-brand-foreground shadow-button hover:from-brand-500 hover:to-indigo-500 hover:shadow-button-hover active:bg-brand-600";
const secondary = "border border-fg/10 bg-bg-subtle/70 text-fg backdrop-blur-xl hover:border-brand-500/30 hover:bg-brand-500/10 active:bg-brand-500/15";
export const button = tv({
    base: "relative inline-flex shrink-0 items-center justify-center gap-2 overflow-hidden rounded-xl font-semibold transition-all duration-200 ease-out select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-400 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-40 disabled:shadow-none aria-disabled:pointer-events-none aria-disabled:opacity-40 active:scale-[0.97]",
    variants: {
        variant: {
            solid: primary,
            primary,
            secondary,
            ghost: "border border-transparent text-fg-muted hover:bg-fg/5 hover:text-fg active:bg-fg/10",
            outline: "border border-brand-500/30 bg-brand-500/10 text-brand-300 hover:border-brand-400/60 hover:bg-brand-500/20",
            danger: "border border-danger/30 bg-danger/10 text-danger hover:bg-danger/20 hover:border-danger/50",
            ghostBrand: "text-brand-400 hover:bg-brand-500/10 hover:text-brand-300",
            microsoft: secondary,
            play: "border border-brand-300/40 bg-gradient-to-br from-brand-400 via-brand-500 to-brand-700 text-brand-foreground shadow-button font-bold hover:from-brand-300 hover:via-brand-400 hover:to-brand-600 hover:shadow-button-hover after:pointer-events-none after:absolute after:inset-0 after:rounded-[inherit] after:border-t after:border-brand-foreground/20"
        },
        size: {
            sm: "h-9 px-3 text-xs",
            md: "h-10 px-4 text-xs",
            lg: "h-11 px-5 text-sm",
            xl: "h-12 px-6 text-sm",
            hero: "h-16 px-8 text-sm tracking-wide",
            icon: "h-10 w-10 p-0"
        },
        block: { true: "w-full" }
    },
    defaultVariants: { variant: "solid", size: "md" }
});
export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
