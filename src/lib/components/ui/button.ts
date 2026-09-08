import { type VariantProps, tv } from "tailwind-variants";

export const button = tv({
	base: "inline-flex items-center justify-center gap-2.5 font-medium tracking-[0.015em] transition-all duration-200 select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed hover:scale-[1.02] active:scale-95 shadow-sm",
	variants: {
		variant: {
			solid: "relative overflow-hidden font-bold rounded-full border border-emerald-400/40 bg-gradient-to-b from-emerald-500 to-teal-700 text-white shadow-[0_4px_16px_rgba(16,185,129,0.35),inset_0_1px_0_rgba(255,255,255,0.3)] hover:from-emerald-400 hover:to-teal-600 hover:border-emerald-300 hover:shadow-[0_6px_22px_rgba(16,185,129,0.5),inset_0_1px_0_rgba(255,255,255,0.4)] active:from-teal-700 active:to-emerald-800 focus-visible:ring-emerald-400/60",
			secondary: "font-semibold rounded-full border border-white/[0.12] bg-[#1a1e29]/90 text-white/90 shadow-[0_2px_8px_rgba(0,0,0,0.25),inset_0_1px_0_rgba(255,255,255,0.06)] hover:bg-[#222838] hover:border-white/[0.22] hover:text-white active:bg-[#151821] focus-visible:ring-emerald-400/40",
			ghost: "rounded-full text-fg-muted bg-transparent hover:bg-white/[0.08] hover:text-white active:bg-white/[0.04] focus-visible:ring-emerald-400/30",
			outline: "font-medium rounded-full border border-emerald-500/35 bg-emerald-500/[0.04] text-emerald-300 hover:bg-emerald-500/[0.12] hover:border-emerald-400/60 hover:text-emerald-200 focus-visible:ring-emerald-400/40",
			danger: "font-semibold rounded-full border border-rose-500/40 bg-gradient-to-b from-rose-600 to-red-700 text-white shadow-[0_4px_14px_rgba(225,29,72,0.35),inset_0_1px_0_rgba(255,255,255,0.25)] hover:from-rose-500 hover:to-red-600 hover:shadow-[0_6px_20px_rgba(225,29,72,0.5)] active:from-red-700 active:to-rose-800 focus-visible:ring-rose-400/50",
			ghostBrand: "font-medium rounded-full text-emerald-400 bg-transparent hover:bg-emerald-500/15 hover:text-emerald-300 active:bg-emerald-500/25 focus-visible:ring-emerald-400/50",
			play: "font-black uppercase tracking-wider rounded-full border-t-2 border-white/35 border-b-2 border-emerald-900/60 bg-gradient-to-r from-emerald-500 via-teal-500 to-emerald-600 text-white shadow-[0_8px_28px_rgba(16,185,129,0.45),inset_0_2px_0_rgba(255,255,255,0.35)] hover:from-emerald-400 hover:via-teal-400 hover:to-emerald-500 hover:shadow-[0_12px_36px_rgba(16,185,129,0.65),inset_0_2px_0_rgba(255,255,255,0.5)] active:shadow-[0_4px_14px_rgba(16,185,129,0.3)] focus-visible:ring-emerald-400/70"
		},
		size: {
			sm: "h-8 px-4 text-xs rounded-full",
			md: "h-10 px-5 text-sm rounded-full",
			lg: "h-12 px-7 text-base rounded-full font-bold",
			xl: "h-14 px-9 text-lg rounded-full font-black",
			hero: "h-16 px-10 text-xl rounded-full font-black",
			icon: "h-10 w-10 p-0 rounded-full"
		},
		block: { true: "w-full" }
	},
	defaultVariants: { variant: "solid", size: "md" }
});

export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
