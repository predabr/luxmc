import { type VariantProps, tv } from "tailwind-variants";

const primary = "launcher-button--primary border border-transparent bg-brand-500 hover:bg-brand-400 active:bg-brand-600 text-brand-foreground rounded-sm";
const secondary = "launcher-button--secondary border border-fg/[0.14] bg-fg/[0.1] hover:bg-fg/[0.16] active:bg-fg/[0.2] text-fg";
const brandOutline = "launcher-button--brand-outline border border-brand-500/60 bg-brand-500/[0.08] text-brand-400 hover:bg-brand-500/[0.16] active:bg-brand-500/[0.22] rounded-sm";

export const button = tv({
    base: "launcher-button relative inline-flex shrink-0 items-center justify-center gap-2 overflow-hidden rounded-sm font-sans font-semibold leading-none whitespace-nowrap transition-[background-color,border-color,color,box-shadow] duration-150 ease-out touch-manipulation select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-400 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 motion-reduce:transition-none",
    variants: {
        variant: {
            solid: primary,
            primary,
            secondary,
            ghost: "launcher-button--ghost border border-transparent text-fg-muted hover:bg-fg/[0.07] hover:text-fg active:bg-fg/[0.11]",
            outline: "launcher-button--outline border border-fg/[0.14] bg-transparent hover:bg-fg/[0.07] text-fg",
            brandOutline,
            ghostDanger: "border border-transparent text-danger hover:bg-danger/10 hover:border-danger/20",
            danger: "launcher-button--danger border border-danger/25 bg-danger/10 text-danger hover:bg-danger/[0.18]",
            ghostBrand: "text-brand-400 hover:bg-brand-500/10 hover:text-brand-300",
            tinted: "border border-transparent bg-brand-500/[0.12] text-brand-400 hover:bg-brand-500/[0.17]",
            microsoft: "launcher-button--microsoft border border-fg/[0.22] bg-fg/[0.1] text-fg hover:bg-fg/[0.16]",
            play: "launcher-button--play border border-transparent bg-brand-500 hover:bg-brand-400 active:bg-brand-600 text-brand-foreground rounded-sm"
        },
        size: {
            sm: "h-10 px-3.5 text-xs rounded-xl",
            md: "h-9 px-4 text-[13px]",
            lg: "h-12 px-5 text-sm rounded-xl",
            xl: "h-11 px-6 text-sm",
            pill: "h-9 px-4 text-xs",
            hero: "h-12 px-8 text-sm",
            icon: "h-9 w-9 p-0",
            iconSm: "h-8 w-8 p-0"
        },
        block: { true: "w-full" }
    },
    defaultVariants: { variant: "secondary", size: "md" }
});
export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
