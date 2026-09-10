import { type VariantProps, tv } from "tailwind-variants";

export const button = tv({
	base: "relative inline-flex items-center justify-center gap-2 font-semibold tracking-normal transition-all duration-200 select-none cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[#caa97c]/50 disabled:pointer-events-none disabled:opacity-40 disabled:cursor-not-allowed active:scale-[0.97] overflow-hidden",
	variants: {
		variant: {
			solid: "font-bold rounded-xl bg-gradient-to-b from-[#ddbe93] to-[#caa97c] text-black shadow-sm hover:brightness-105 active:brightness-95 border border-[#ebd095]/40 hover:shadow-md hover:shadow-[#caa97c]/20 transition-all",
			primary: "font-bold rounded-xl bg-[#6c5ce7] hover:bg-[#5b4cdb] text-white shadow-md shadow-[#6c5ce7]/25 active:bg-[#4834d4] hover:-translate-y-0.5 transition-all",
			secondary: "font-semibold rounded-xl border border-white/10 bg-[#1e1f26]/80 backdrop-blur-md text-white/90 shadow-sm hover:bg-[#282a34] hover:text-white hover:border-white/20 hover:shadow-md transition-all",
			ghost: "font-semibold rounded-xl text-white/70 bg-transparent hover:bg-white/10 hover:text-white active:bg-white/5 transition-all",
			outline: "font-semibold rounded-xl border border-[#caa97c]/40 bg-[#caa97c]/10 text-[#caa97c] hover:bg-[#caa97c]/20 hover:border-[#caa97c]/80 hover:text-white hover:shadow-sm hover:shadow-[#caa97c]/15 transition-all",
			danger: "font-semibold rounded-xl border border-red-500/30 bg-red-500/15 text-red-400 hover:bg-red-500/25 hover:border-red-500/60 hover:text-red-200 shadow-sm active:bg-red-500/35 transition-all",
			ghostBrand: "font-semibold rounded-xl text-[#caa97c] bg-transparent hover:bg-[#caa97c]/10 hover:text-white active:bg-[#caa97c]/20 transition-all",
			microsoft: "font-bold rounded-2xl bg-[#1e1f26] hover:bg-[#262832] border border-white/15 text-white shadow-lg hover:border-white/30 hover:-translate-y-0.5 hover:shadow-xl transition-all",
			play: "font-black uppercase tracking-wider rounded-2xl bg-gradient-to-r from-[#caa97c] via-[#ddbe93] to-[#ebd095] text-black shadow-lg shadow-[#caa97c]/25 hover:shadow-[#caa97c]/40 hover:brightness-105 active:scale-[0.97] hover:-translate-y-0.5 border border-[#fff]/30 transition-all"
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
