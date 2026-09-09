import { type VariantProps, tv } from "tailwind-variants";

export const button = tv({
	base: "inline-flex items-center justify-center gap-2.5 font-bold tracking-[0.015em] transition-all duration-200 select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-bg disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed hover:scale-[1.03] active:scale-95 shadow-md",
	variants: {
		variant: {
			solid: "relative overflow-hidden font-black rounded-full border border-amber-300/40 bg-gradient-to-r from-[#e2b86b] via-[#f0cb85] to-[#e2b86b] text-black shadow-[0_4px_20px_rgba(226,184,107,0.4)] hover:brightness-110 hover:shadow-[0_6px_28px_rgba(226,184,107,0.6)] focus-visible:ring-brand-500/60",
			secondary: "font-bold rounded-full border border-white/10 bg-[#1c1d22] text-white/90 shadow-[0_2px_12px_rgba(0,0,0,0.3)] hover:bg-[#25262c] hover:border-brand-500/40 hover:text-white focus-visible:ring-brand-500/40",
			ghost: "rounded-full text-white/70 bg-transparent hover:bg-white/10 hover:text-white active:bg-white/5 focus-visible:ring-brand-500/30",
			outline: "font-bold rounded-full border border-brand-500/40 bg-brand-500/10 text-brand-500 hover:bg-brand-500/20 hover:border-brand-500 hover:text-[#f3d79e] focus-visible:ring-brand-500/40",
			danger: "font-bold rounded-full border border-rose-500/40 bg-gradient-to-r from-rose-600 to-red-700 text-white shadow-[0_4px_16px_rgba(225,29,72,0.35)] hover:from-rose-500 hover:to-red-600 hover:shadow-[0_6px_22px_rgba(225,29,72,0.5)] active:from-red-700 active:to-rose-800 focus-visible:ring-rose-400/50",
			ghostBrand: "font-bold rounded-full text-brand-500 bg-transparent hover:bg-brand-500/15 hover:text-[#f3d79e] active:bg-brand-500/25 focus-visible:ring-brand-500/50",
			play: "font-black uppercase tracking-wider rounded-full border border-amber-300/50 bg-gradient-to-r from-[#e2b86b] via-[#ebd095] to-[#e2b86b] text-black shadow-[0_0_25px_rgba(226,184,107,0.5)] hover:scale-105 hover:brightness-110 active:scale-95 focus-visible:ring-brand-500/70"
		},
		size: {
			sm: "h-8 px-4 text-xs rounded-full",
			md: "h-10 px-5 text-xs rounded-full",
			lg: "h-12 px-7 text-sm rounded-full font-bold",
			xl: "h-14 px-9 text-base rounded-full font-black",
			hero: "h-16 px-10 text-lg rounded-full font-black",
			icon: "h-10 w-10 p-0 rounded-full"
		},
		block: { true: "w-full" }
	},
	defaultVariants: { variant: "solid", size: "md" }
});

export type ButtonVariant = VariantProps<typeof button>["variant"];
export type ButtonSize = VariantProps<typeof button>["size"];
