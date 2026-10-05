<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { toast } from "$lib/stores/toasts.svelte";
    import { openPortal } from "$lib/api/deepLinks";
	import { Sun, Moon, Globe, Gauge, Zap, ExternalLink, Sparkles } from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { setLocale } from "$lib/stores/persistence.svelte";
	import { appState } from "$lib/stores/app.svelte";
	import { updaterStore } from "$lib/stores/updater.svelte";
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
		settings.patch({ performanceMode: appState.performanceMode });
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
				<span class="text-[10px] uppercase font-bold text-brand-300">{uiText("ui.d04180ac10dfe21c")}</span>
			</div>
		{/if}
	</div>

	<div class="flex items-center gap-2 ml-auto">
		{#if updaterStore.updateAvailable}
			<button
				type="button"
				onclick={() => {
					updaterStore.showModal = true;
				}}
				class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2 animate-pulse" })}
				title={uiText("ui.1192fb8d62671020", {arg0: (updaterStore.latestVersion)})}
			>
				<Sparkles class="h-3.5 w-3.5 text-emerald-300" />
				<span>{uiText("ui.5b6c369bff15e1a8")}{updaterStore.latestVersion} {uiText("ui.113b928fd873626a")}</span>
			</button>
		{/if}

		<button
			onclick={togglePerformance}
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-2" })}
			style={appState.performanceMode ? "background: rgb(var(--danger) / 0.15); border-color: rgb(var(--danger) / 0.3); color: rgb(var(--danger));" : "background: rgb(var(--fg) / 0.05); border-color: transparent; color: rgb(var(--fg) / 0.6);"}
			title={uiText("ui.9edb965dc1f0e59e")}
		>
			{#if appState.performanceMode}
				<Zap class="h-4 w-4" />
				{uiText("ui.dc440f158581aa5e")}
			{:else}
				<Gauge class="h-4 w-4" />
				{uiText("ui.c89982e1ed8c1a97")}
			{/if}
		</button>

		<button
			type="button"
			onclick={() => { void openPortal().catch(error => toast(String(error), "error")); }}
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
			title={uiText("ui.701927621c7b7dbf")}
		>
			<Globe class="h-3.5 w-3.5 text-emerald-400" />
			<span>{uiText("ui.58acfa2d9f26f747")}</span>
			<ExternalLink class="h-3 w-3 text-fg/40" />
		</button>

		<div class="h-5 w-px bg-fg/10 mx-1"></div>

		<button
			onclick={toggleLanguage}
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center gap-1.5" })}
			title={settings.value.language === "en" ? "English" : settings.value.language === "es" ? "Español" : uiText("ui.d4ada8ec276411a1")}
		>
			<Globe class="h-4 w-4" />
			<span>{settings.value.language === "en" ? "EN" : settings.value.language === "es" ? "ES" : "PT"}</span>
		</button>

		<button
			onclick={cycleTheme}
			class={launcherButton({ variant: "secondary", size: "sm", class: "flex items-center justify-center" })}
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
