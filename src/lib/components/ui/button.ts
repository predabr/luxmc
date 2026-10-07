import { type VariantProps, tv } from "tailwind-variants";

const primary = "launcher-button--primary border border-brand-300/35 bg-brand-500/90 hover:bg-brand-400/95 active:bg-brand-600/95 text-brand-foreground shadow-button";
const secondary = "launcher-button--secondary border border-fg/[0.12] bg-fg/[0.045] hover:bg-fg/[0.09] text-fg hover:border-fg/25";

export const button = tv({
    base: "launcher-button relative inline-flex shrink-0 items-center justify-center gap-2.5 overflow-hidden rounded-lg font-sans font-semibold leading-none transition-[background-color,border-color,box-shadow,transform] duration-150 ease-out touch-manipulation select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-400 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-50 disabled:shadow-none aria-disabled:pointer-events-none aria-disabled:opacity-50 motion-reduce:transition-none",
    variants: {
        variant: {
            solid: primary,
            primary,
            secondary,
            ghost: "launcher-button--ghost border border-transparent text-fg-muted hover:bg-fg/5 hover:text-fg active:bg-fg/10",
            outline: "launcher-button--outline border border-fg/15 bg-fg/[0.025] text-fg hover:border-brand-400/40 hover:bg-fg/[0.075]",
            ghostDanger: "border border-transparent text-danger hover:bg-danger/10 hover:border-danger/20",
            danger: "launcher-button--danger border border-danger/30 bg-danger/10 text-danger hover:bg-danger/20 hover:border-danger/50",
            ghostBrand: "text-brand-400 hover:bg-brand-500/10 hover:text-brand-300",
            microsoft: "launcher-button--microsoft border border-fg/20 bg-fg/[0.07] text-fg hover:border-fg/40 hover:bg-fg/10 shadow-soft",
            play: "launcher-button--play border border-brand-300/40 bg-gradient-to-r from-brand-600 via-brand-500 to-brand-400 text-brand-foreground font-bold tracking-wide shadow-button-hover hover:brightness-110"
        },
        size: {
            sm: "h-10 px-3.5 text-xs rounded-xl",
            md: "h-11 px-4 text-[13px] rounded-xl",
            lg: "h-12 px-5 text-sm rounded-xl",
            xl: "h-14 px-6 text-sm rounded-2xl",
            pill: "h-10 px-5 text-xs rounded-full",
            hero: "h-14 px-8 text-sm tracking-wide rounded-2xl",
            icon: "h-11 w-11 p-0 rounded-xl"
        },
        block: { true: "w-full" }
    },
    defaultVariants: { variant: "secondary", size: "md" }
});
export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];

