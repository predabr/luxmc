import { type VariantProps, tv } from "tailwind-variants";

export const button = tv({
	base: "relative inline-flex items-center justify-center gap-2 font-semibold tracking-normal transition-all duration-200 ease-out select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500/50 disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:shadow-none active:scale-[0.97] overflow-hidden",
	variants: {
		variant: {
			solid: "font-bold rounded-2xl bg-gradient-to-b from-brand-400 via-brand-500 to-brand-600 text-brand-foreground shadow-[0_4px_14px_-3px_theme(colors.brand.600/0.5),inset_0_1px_0_theme(colors.white/0.2)] border border-brand-400/50 hover:from-brand-300 hover:via-brand-400 hover:to-brand-500 hover:shadow-[0_8px_25px_-5px_theme(colors.brand.500/0.45),inset_0_1px_0_theme(colors.white/0.25)] hover:-translate-y-0.5 active:from-brand-600 active:via-brand-600 active:to-brand-700 active:shadow-[0_2px_8px_-2px_theme(colors.brand.600/0.4)] active:translate-y-0 after:absolute after:inset-0 after:rounded-[inherit] after:bg-gradient-to-r after:from-transparent after:via-white/20 after:to-transparent after:-translate-x-full after:transition-transform after:duration-500 after:ease-out hover:after:translate-x-full",
			primary: "font-bold rounded-2xl bg-gradient-to-b from-brand-500 to-brand-700 text-brand-foreground shadow-[0_3px_12px_-4px_theme(colors.brand.700/0.5),inset_0_1px_0_theme(colors.white/0.15)] border border-brand-500/30 hover:from-brand-400 hover:to-brand-600 hover:shadow-[0_6px_20px_-4px_theme(colors.brand.600/0.4)] hover:-translate-y-0.5 active:from-brand-600 active:to-brand-800 active:shadow-sm active:translate-y-0",
			secondary: "font-semibold rounded-2xl border border-white/10 bg-gradient-to-b from-bg-elevated/90 to-bg-elevated/70 backdrop-blur-md text-fg/90 shadow-[0_2px_8px_-2px_theme(colors.black/0.25),inset_0_1px_0_theme(colors.white/0.06)] hover:from-bg-subtle hover:to-bg-elevated hover:text-fg hover:border-white/20 hover:shadow-[0_4px_16px_-3px_theme(colors.black/0.3)] hover:-translate-y-0.5 active:from-bg-base active:to-bg-subtle active:shadow-sm active:translate-y-0",
			ghost: "font-semibold rounded-2xl text-fg-muted bg-transparent hover:bg-white/10 hover:text-fg active:bg-white/5",
			outline: "font-semibold rounded-2xl border-2 border-brand-500/40 bg-brand-500/10 text-brand-300 hover:bg-brand-500/20 hover:border-brand-500/70 hover:text-fg hover:shadow-[0_4px_14px_-3px_theme(colors.brand.500/0.2)] active:bg-brand-500/30 active:shadow-sm",
			danger: "font-semibold rounded-2xl bg-gradient-to-b from-danger/80 to-danger text-white shadow-[0_3px_10px_-3px_theme(colors.danger/0.4),inset_0_1px_0_theme(colors.white/0.15)] border border-danger/30 hover:from-danger/90 hover:to-danger hover:shadow-[0_6px_18px_-3px_theme(colors.danger/0.45)] hover:-translate-y-0.5 active:from-danger active:to-red-700 active:shadow-sm active:translate-y-0",
			ghostBrand: "font-semibold rounded-2xl text-brand-400 bg-transparent hover:bg-brand-500/10 hover:text-brand-300 active:bg-brand-500/20",
			microsoft: "font-bold rounded-2xl bg-gradient-to-b from-bg-elevated to-bg-elevated/80 hover:from-bg-subtle hover:to-bg-elevated border border-white/15 text-fg shadow-[0_4px_16px_-4px_theme(colors.black/0.35),inset_0_1px_0_theme(colors.white/0.08)] hover:border-white/30 hover:shadow-[0_8px_25px_-5px_theme(colors.black/0.4)] hover:-translate-y-0.5 active:from-bg-base active:to-bg-subtle active:shadow-sm active:translate-y-0",
			play: "font-black uppercase tracking-widest rounded-2xl bg-gradient-to-br from-brand-400 via-brand-500 to-brand-700 text-brand-foreground shadow-[0_6px_30px_-5px_theme(colors.brand.500/0.5),0_2px_8px_-2px_theme(colors.brand.600/0.3),inset_0_2px_0_theme(colors.white/0.2),inset_0_-1px_0_theme(colors.brand.700/0.3)] border border-white/25 hover:from-brand-300 hover:via-brand-400 hover:to-brand-600 hover:shadow-[0_10px_40px_-5px_theme(colors.brand.500/0.55),0_4px_12px_-3px_theme(colors.brand.600/0.3),inset_0_2px_0_theme(colors.white/0.25)] hover:-translate-y-0.5 active:from-brand-500 active:via-brand-600 active:to-brand-800 active:shadow-[0_2px_10px_-3px_theme(colors.brand.600/0.4),inset_0_2px_0_theme(colors.brand.700/0.2)] active:translate-y-0 after:absolute after:inset-0 after:rounded-[inherit] after:bg-gradient-to-r after:from-transparent after:via-white/25 after:to-transparent after:-translate-x-full after:transition-transform after:duration-500 after:ease-out hover:after:translate-x-full"
		},
		size: {
			sm: "h-8 px-3 text-xs rounded-xl",
			md: "h-9 px-4 text-xs rounded-xl",
			lg: "h-11 px-6 text-sm rounded-2xl font-bold",
			xl: "h-12 px-7 text-sm rounded-2xl font-bold",
			hero: "h-14 px-8 text-base rounded-2xl font-extrabold",
			icon: "h-9 w-9 p-0 rounded-xl"
		},
		block: { true: "w-full" }
	},
	defaultVariants: { variant: "solid", size: "md" }
});

export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
