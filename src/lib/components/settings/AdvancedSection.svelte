<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	import { runtimePlatform } from "$lib/stores/platform.svelte";
	import { Zap, Gamepad2 } from "lucide-svelte";
	import { discordSetActivity } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		performanceMode: boolean;
		discordRpc: boolean;
		onPerformanceModeChange: (val: boolean) => void;
		onDiscordRpcChange: (val: boolean) => void;
	};

	let {
		performanceMode,
		discordRpc,
		onPerformanceModeChange,
		onDiscordRpcChange,
	}: Props = $props();

	async function toggleDiscordRpc() {
		const next = !discordRpc;
		onDiscordRpcChange(next);
		if (next) {
			const ok = await discordSetActivity({
				details: uiText("ui.7cccac6544c65648"),
				state: runtimePlatform.label,
				largeText: "Luxmc Launcher",
				largeImage: "https://raw.githubusercontent.com/predabr/luxmc/main/src-tauri/icons/icon.png",
				smallImage: "grass",
				smallText: `Minecraft · ${runtimePlatform.label}`
			});
			if (ok) {
				toast(uiText("ui.3bbe0b4ae4fb2b8d"), "success");
			} else {
				toast(uiText("ui.72db757d4478c417"), "info");
			}
		} else {
			toast("Discord Rich Presence desativado.", "info");
		}
	}
</script>

<div class="space-y-3">
	<!-- Performance Mode -->
	<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3 flex items-center justify-between hover:border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity]">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-fg/5 flex items-center justify-center text-orange-400">
				<Zap class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-fg">{uiText("ui.7cf5d0f1963a5634")}</div>
				<div class="text-[10px] text-fg/40">{uiText("ui.192a4a5f179f5e06")}</div>
			</div>
		</div>
		<button
			type="button"
			role="switch"
			aria-label={uiText("ui.4e4fd99bf0431ab4")}
			aria-checked={performanceMode}
			class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {performanceMode ? 'bg-brand-400' : 'bg-bg-subtle'}"
			onclick={() => onPerformanceModeChange(!performanceMode)}
		>
			<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {performanceMode ? 'translate-x-5 bg-bg-subtle' : 'translate-x-0 bg-fg'}"></span>
		</button>
	</div>

	<!-- Discord RPC -->
	<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3 flex items-center justify-between hover:border-fg/10 transition-[color,background-color,border-color,box-shadow,transform,opacity]">
		<div class="flex items-center gap-3">
			<div class="w-9 h-9 rounded-xl bg-fg/5 flex items-center justify-center text-indigo-400">
				<Gamepad2 class="w-4 h-4" />
			</div>
			<div>
				<div class="text-xs font-bold text-fg">{uiText("settings.discordRpcTitle")}</div>
				<div class="text-[10px] text-fg/40">{uiText("ui.097c7496deda1697")}</div>
			</div>
		</div>
		<button
			type="button"
			role="switch"
			aria-label={uiText("settings.discordRpcTitle")}
			aria-checked={discordRpc}
			class="w-11 h-6 rounded-full transition-colors duration-200 relative flex items-center px-0.5 cursor-pointer shrink-0 {discordRpc ? 'bg-brand-400' : 'bg-bg-subtle'}"
			onclick={toggleDiscordRpc}
		>
			<span class="w-5 h-5 rounded-full transition-transform duration-200 shadow-md {discordRpc ? 'translate-x-5 bg-bg-subtle' : 'translate-x-0 bg-fg'}"></span>
		</button>
	</div>
</div>
