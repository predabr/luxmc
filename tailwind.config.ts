import type { Config } from "tailwindcss";
import forms from "@tailwindcss/forms";
import typography from "@tailwindcss/typography";

const config: Config = {
	darkMode: ["class"],
	content: ["./src/**/*.{html,js,svelte,ts}"],
	safelist: ["dark"],
	theme: {
		container: {
			center: true,
			padding: "1.5rem",
			screens: {
				"2xl": "1400px"
			}
		},
		extend: {
			colors: {
				bg: {
					DEFAULT: "rgb(var(--bg) / <alpha-value>)",
					elevated: "rgb(var(--bg-elevated) / <alpha-value>)",
					subtle: "rgb(var(--bg-subtle) / <alpha-value>)",
					overlay: "rgb(var(--bg-overlay) / <alpha-value>)"
				},
				fg: {
					DEFAULT: "rgb(var(--fg) / <alpha-value>)",
					muted: "rgb(var(--fg-muted) / <alpha-value>)",
					subtle: "rgb(var(--fg-subtle) / <alpha-value>)"
				},
				border: {
					DEFAULT: "rgb(var(--border) / <alpha-value>)",
					strong: "rgb(var(--border-strong) / <alpha-value>)"
				},
				brand: {
					50: "rgb(var(--brand-50) / <alpha-value>)",
					100: "rgb(var(--brand-100) / <alpha-value>)",
					200: "rgb(var(--brand-200) / <alpha-value>)",
					300: "rgb(var(--brand-300) / <alpha-value>)",
					400: "rgb(var(--brand-400) / <alpha-value>)",
					500: "rgb(var(--brand-500) / <alpha-value>)",
					600: "rgb(var(--brand-600) / <alpha-value>)",
					700: "rgb(var(--brand-700) / <alpha-value>)",
					800: "rgb(var(--brand-800) / <alpha-value>)",
					900: "rgb(var(--brand-900) / <alpha-value>)",
					950: "rgb(var(--brand-950) / <alpha-value>)",
					DEFAULT: "rgb(var(--brand-500) / <alpha-value>)",
					foreground: "rgb(var(--brand-foreground) / <alpha-value>)"
				},
				success: "rgb(var(--success) / <alpha-value>)",
				warning: "rgb(var(--warning) / <alpha-value>)",
				danger: "rgb(var(--danger) / <alpha-value>)",
				info: "rgb(var(--info) / <alpha-value>)"
			},
			borderRadius: {
				xs: "var(--radius-xs)",
				sm: "var(--radius-sm)",
				md: "var(--radius-md)",
				lg: "var(--radius-lg)",
				xl: "var(--radius-xl)",
				"2xl": "var(--radius-2xl)"
			},
			fontFamily: {
				sans: [
					"Inter",
					"ui-sans-serif",
					"system-ui",
					"-apple-system",
					"Segoe UI",
					"Roboto",
					"sans-serif"
				],
				mono: [
					"JetBrains Mono",
					"ui-monospace",
					"SFMono-Regular",
					"Menlo",
					"monospace"
				]
			},
			boxShadow: {
				soft: "var(--shadow-soft)",
				elevated: "var(--shadow-elevated)",
				glow: "0 0 30px -5px rgb(var(--brand-500) / 0.45)"
			},
			keyframes: {
				"fade-in": {
					"0%": { opacity: "0" },
					"100%": { opacity: "1" }
				},
				"slide-up": {
					"0%": { transform: "translateY(8px)", opacity: "0" },
					"100%": { transform: "translateY(0)", opacity: "1" }
				},
				shimmer: {
					"0%": { backgroundPosition: "-1000px 0" },
					"100%": { backgroundPosition: "1000px 0" }
				},
				"btn-shimmer": {
					"0%": { left: "-100%" },
					"100%": { left: "200%" }
				}
			},
			animation: {
				"fade-in": "fade-in 200ms ease-out",
				"slide-up": "slide-up 200ms ease-out",
				shimmer: "shimmer 2s linear infinite",
				"btn-shimmer": "btn-shimmer 0.6s ease-out"
			}
		}
	},
	plugins: [forms({ strategy: "class" }), typography]
};

export default config;
