<script lang="ts">
	import { Sun, Moon, Globe, Gauge, Zap } from "lucide-svelte";
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
		const next = settings.value.language === "en" ? "pt-BR" : "en";
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
				<span class="text-white/60 font-medium">Logged in as</span>
				<span class="font-bold text-white drop-shadow-md">{account.value.username}</span>
			</div>
		{:else}
			<div class="flex flex-col gap-0.5">
				<span class="text-sm font-bold text-white tracking-tight">Luxmc</span>
				<span class="text-[10px] uppercase font-bold text-brand-300">Ultimate Edition</span>
			</div>
		{/if}
	</div>

	<div class="flex items-center gap-2 ml-auto">
		<button
			onclick={togglePerformance}
			class="flex h-9 items-center gap-2 px-3 rounded-xl text-xs font-bold transition-all duration-200 border"
			style={appState.performanceMode ? "background: rgba(239, 68, 68, 0.15); border-color: rgba(239, 68, 68, 0.3); color: rgb(248, 113, 113);" : "background: rgba(255, 255, 255, 0.05); border-color: transparent; color: rgba(255, 255, 255, 0.6);"}
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

		<div class="h-5 w-px bg-white/10 mx-1"></div>

		<button
			onclick={toggleLanguage}
			class="flex h-9 items-center gap-1.5 px-2.5 rounded-lg text-xs font-bold transition-all duration-200 hover:bg-white/10 text-white/60 hover:text-white"
			title={settings.value.language === "en" ? t("settings.switchToPortuguese") : t("settings.switchToEnglish")}
		>
			<Globe class="h-4 w-4" />
			<span>{settings.value.language === "en" ? "EN" : "PT"}</span>
		</button>

		<button
			onclick={cycleTheme}
			class="flex h-9 w-9 items-center justify-center rounded-lg transition-all duration-200 hover:bg-white/10 text-white/60 hover:text-white"
		>
			{#if settings.value.theme === "default-dark"}
				<Moon class="h-4 w-4" />
			{:else}
				<Sun class="h-4 w-4" />
			{/if}
		</button>

		<div class="h-5 w-px bg-white/10 mx-1"></div>

		<div class="ml-1">
			<AccountIndicator />
		</div>
	</div>
</header>
