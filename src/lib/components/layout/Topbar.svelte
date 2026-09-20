<script lang="ts">
    import { toast } from "$lib/stores/toasts.svelte";
    import { openPortal } from "$lib/api/deepLinks";
	import { Sun, Moon, Globe, Gauge, Zap, ExternalLink } from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { setLocale } from "$lib/stores/persistence.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { useTranslation } from "$lib/i18n/useTranslation.svelte";
	import AccountIndicator from "$lib/components/ui/AccountIndicator.svelte";

	const { t } = useTranslation();

	function cycleTheme() {
		const isDark = settings.value.theme === "default-dark";
		settings.patch({ theme: isDark ? "default-light" : "default-dark" });
	}

	function toggleLanguage() {
		const current = settings.value.language;
		const next = current === "en" ? "pt-BR" : current === "pt-BR" ? "es" : "en";
		void setLocale(next);
	}
	
	function togglePerformance() {
		appState.performanceMode = !appState.performanceMode;
	}
</script>

<header class="flex h-14 shrink-0 items-center justify-between px-6 gap-4 border-b luxmc-glass z-10">
	<div class="flex items-center gap-3 min-w-0">
		{#if account.value}
			<div class="flex items-center gap-2.5 text-sm">
				<span class="text-fg/60 font-medium">{t("home.playingAs") || "Logged in as"}</span>
				<span class="font-bold text-fg drop-shadow-md">{account.value.username}</span>
			</div>
		{:else}
			<div class="flex flex-col gap-0.5">
				<span class="text-sm font-bold text-fg tracking-tight">Luxmc</span>
				<span class="text-[10px] uppercase font-bold text-brand-300">Ultimate Edition</span>
			</div>
		{/if}
	</div>

	<div class="flex items-center gap-2 ml-auto">
		<button
			onclick={togglePerformance}
			class="flex h-9 items-center gap-2 px-3 rounded-xl text-xs font-bold transition-all duration-200 border"
			style={appState.performanceMode ? "background: rgb(var(--danger) / 0.15); border-color: rgb(var(--danger) / 0.3); color: rgb(var(--danger));" : "background: rgb(var(--fg) / 0.05); border-color: transparent; color: rgb(var(--fg) / 0.6);"}
			title="Toggle Performance Mode"
		>
			{#if appState.performanceMode}
				<Zap class="h-4 w-4" />
				PERFORMANCE ON
			{:else}
				<Gauge class="h-4 w-4" />
				PERFORMANCE OFF
			{/if}
		</button>

		<button
			type="button"
			onclick={() => { void openPortal().catch(error => toast(String(error), "error")); }}
			class="flex h-9 items-center gap-1.5 px-3 rounded-xl text-xs font-bold transition-all duration-200 bg-fg/5 hover:bg-fg/10 hover:border-emerald-500/30 border border-fg/5 text-fg/80 hover:text-fg cursor-pointer active:scale-[0.98] shadow-sm"
			title="Abrir Portal Web & Studio 3D (luxmc-r92.pages.dev)"
		>
			<Globe class="h-3.5 w-3.5 text-emerald-400" />
			<span>Site & 3D</span>
			<ExternalLink class="h-3 w-3 text-fg/40" />
		</button>

		<div class="h-5 w-px bg-fg/10 mx-1"></div>

		<button
			onclick={toggleLanguage}
			class="flex h-9 items-center gap-1.5 px-2.5 rounded-lg text-xs font-bold transition-all duration-200 hover:bg-fg/10 text-fg/60 hover:text-fg"
			title={settings.value.language === "en" ? "English" : settings.value.language === "es" ? "Español" : "Português"}
		>
			<Globe class="h-4 w-4" />
			<span>{settings.value.language === "en" ? "EN" : settings.value.language === "es" ? "ES" : "PT"}</span>
		</button>

		<button
			onclick={cycleTheme}
			class="flex h-9 w-9 items-center justify-center rounded-lg transition-all duration-200 hover:bg-fg/10 text-fg/60 hover:text-fg"
		>
			{#if settings.value.theme === "default-dark"}
				<Moon class="h-4 w-4" />
			{:else}
				<Sun class="h-4 w-4" />
			{/if}
		</button>

		<div class="h-5 w-px bg-fg/10 mx-1"></div>

		<div class="ml-1">
			<AccountIndicator />
		</div>
	</div>
</header>
