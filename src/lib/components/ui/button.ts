import { type VariantProps, tv } from "tailwind-variants";

export const button = tv({
	base: "inline-flex items-center justify-center gap-2 font-semibold tracking-normal transition-all duration-150 select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500/50 disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed active:scale-[0.98]",
	variants: {
		variant: {
			solid: "font-bold rounded-xl bg-[#caa97c] hover:bg-[#d8bc98] text-[#15171c] shadow-sm hover:brightness-105 active:brightness-95",
			primary: "font-bold rounded-xl bg-[#6c5ce7] hover:bg-[#5b4cdb] text-white shadow-md shadow-[#6c5ce7]/25 active:bg-[#4834d4]",
			secondary: "font-semibold rounded-xl border border-white/10 bg-[#1e1f24] text-white/90 shadow-sm hover:bg-[#282930] hover:text-white hover:border-white/20",
			ghost: "font-semibold rounded-xl text-white/70 bg-transparent hover:bg-white/10 hover:text-white active:bg-white/5",
			outline: "font-semibold rounded-xl border border-[#caa97c]/40 bg-[#caa97c]/10 text-[#caa97c] hover:bg-[#caa97c]/20 hover:border-[#caa97c] hover:text-white",
			danger: "font-semibold rounded-xl bg-red-600 hover:bg-red-500 text-white shadow-sm active:bg-red-700",
			ghostBrand: "font-semibold rounded-xl text-[#caa97c] bg-transparent hover:bg-[#caa97c]/10 hover:text-white active:bg-[#caa97c]/20",
			microsoft: "font-bold rounded-2xl bg-[#1e1f26] hover:bg-[#262832] border border-white/15 text-white shadow-lg hover:border-white/30 transition-all",
			play: "font-black uppercase tracking-wider rounded-2xl bg-gradient-to-r from-[#cfb491] to-[#bda079] text-[#121316] shadow-lg hover:brightness-105 active:scale-[0.98] transition-all"
		},
		size: {
			sm: "h-8 px-3 text-xs rounded-xl",
			md: "h-9 px-4 text-xs rounded-xl",
			lg: "h-11 px-6 text-sm rounded-xl font-bold",
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
