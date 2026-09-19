<script lang="ts">
    import { luxAccountLogout } from "$lib/api/luxAccount";
    import { cloudAccount } from "$lib/stores/cloudAccount.svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		User, 
		LogOut, 
		Check, 
		X, 
		ShieldCheck, 
		Sparkles, 
		Gamepad2, 
		Shirt, 
		Clock, 
		Layers, 
		Zap,
		Pencil,
		AlertCircle
	} from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { activeSkinStore } from "$lib/stores/skin.svelte";
	import { gamingStats } from "$lib/stores/gamingStats.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { goto } from "$app/navigation";

	let { onClose }: { onClose: () => void } = $props();

	let currentUsername = $derived(account.value?.username || "Gamer");
	let isEditingNick = $state(false);
	let editedNick = $state(account.value?.username || "");
	let isMicrosoft = $derived(Boolean(account.value?.minecraftToken && account.value.minecraftToken.length > 30));

	function handleSaveNick() {
        if (account.value?.id.startsWith("luxmc:")) { toast("O nickname identifica sua conta Luxmc e não pode ser alterado localmente.", "info"); return; }
		const trimmed = editedNick.trim();
		if (!trimmed) {
			toast("O nickname não pode ser vazio.", "error");
			return;
		}
		if (trimmed.length < 3 || trimmed.length > 16) {
			toast("O nickname deve ter entre 3 e 16 caracteres.", "error");
			return;
		}
		if (account.value) {
			const updated = {
				...account.value,
				username: trimmed
			};
			account.value = updated;
			localStorage.setItem("luxmc_current_account", JSON.stringify(updated));
		}
		isEditingNick = false;
		toast(`Nickname alterado para ${trimmed}!`, "success");
	}

	async function handleLogout() {
        if (account.value?.id.startsWith("luxmc:")) {
            try { await luxAccountLogout(account.value.id); } catch (cause) { toast(String(cause), "error"); return; }
        }
		account.clear();
		localStorage.removeItem("luxmc_current_account");
		onClose();
		toast("Sessão encerrada com sucesso. Até logo!", "info");
		goto("/");
	}

	function handleNavigateSkins() {
		onClose();
		goto("/skins");
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div 
	class="fixed inset-0 z-[10000] flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
	in:fade={{ duration: 200 }}
	out:fade={{ duration: 150 }}
	onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
	role="dialog"
	aria-modal="true"
	tabindex="-1"
>
	<!-- Modal Card -->
	<div 
		class="w-full max-w-md rounded-3xl bg-bg-elevated border border-fg/10 shadow-elevated overflow-hidden flex flex-col"
		in:scale={{ start: 0.95, duration: 220 }}
		out:scale={{ start: 0.95, duration: 150 }}
	>
        {#if account.value?.id.startsWith("luxmc:")}
            <div class="mx-5 my-3 rounded-xl border border-brand-500/20 bg-brand-500/5 p-3 text-xs">
                <p class="font-semibold text-brand-400">Conta Luxmc · {cloudAccount.busy ? "Sincronizando…" : cloudAccount.ready ? "Sincronização ativa" : "Conectando…"}</p>
                {#if cloudAccount.error}<p role="alert" class="mt-2 text-danger">{cloudAccount.error}</p><button type="button" class="mt-2 text-brand-400" onclick={() => cloudAccount.reload()}>Carregar preferências do site novamente</button>{/if}
            </div>
        {/if}
		<!-- Header Banner -->
		<div class="h-28 w-full relative bg-gradient-to-r from-amber-500/20 via-purple-500/20 to-blue-500/20 p-5 flex items-start justify-between">
			<div class="flex items-center gap-2">
				<span class="bg-bg-overlay/60 backdrop-blur-md border border-fg/10 text-fg/80 text-[10px] font-black uppercase px-3 py-1 rounded-full flex items-center gap-1.5">
					<Zap class="w-3 h-3 text-brand-500" /> Perfil do Jogador
				</span>
			</div>
			
			<button 
				type="button"
				class="h-8 w-8 rounded-full bg-bg-overlay/40 hover:bg-fg/10 text-fg/70 hover:text-fg flex items-center justify-center transition-all cursor-pointer"
				onclick={onClose}
				title="Fechar"
			>
				<X class="w-4 h-4" />
			</button>
		</div>

		<!-- Avatar & User Info Row -->
		<div class="px-6 pb-6 pt-0 relative flex flex-col">
			<!-- Overlapping Avatar -->
			<div class="-mt-12 mb-4 flex items-end justify-between">
				<div class="relative">
					<div class="h-20 w-20 rounded-full overflow-hidden bg-bg-subtle border-4 border-border shadow-2xl flex items-center justify-center">
						<img 
							src={activeSkinStore.current.avatarUrl || (account.value ? "https://mc-heads.net/avatar/" + account.value.uuid + "/100" : "https://mc-heads.net/avatar/MHF_Steve/100")} 
							alt="Avatar" 
							class="w-full h-full object-cover"
						/>
					</div>
					<div class="absolute -bottom-1 -right-1 h-5 w-5 bg-emerald-500 rounded-full border-2 border-border shadow-md flex items-center justify-center">
						<div class="h-2 w-2 rounded-full bg-fg animate-pulse"></div>
					</div>
				</div>

				<button 
					type="button"
					class="px-4 py-2 rounded-full bg-fg/5 hover:bg-fg/10 border border-fg/10 text-xs font-bold text-fg flex items-center gap-1.5 transition-all active:scale-[0.98] cursor-pointer"
					onclick={handleNavigateSkins}
				>
					<Shirt class="w-3.5 h-3.5 text-brand-500" />
					Trocar Skin
				</button>
			</div>

			<!-- Nickname & Account Type -->
			<div class="space-y-1">
				{#if isEditingNick}
					<div class="flex items-center gap-2 mt-1">
						<input 
							type="text" 
							bind:value={editedNick} 
							class="flex-1 bg-bg-subtle border border-brand-500/50 rounded-full px-4 py-2 text-sm font-bold text-fg outline-none focus:ring-1 focus:ring-brand-500"
							placeholder="Novo nickname"
							maxlength="16"
							onkeydown={(e) => { if (e.key === "Enter") handleSaveNick(); }}
						/>
						<button 
							type="button"
							class="h-9 px-4 rounded-full bg-brand-500 hover:bg-brand-400 text-brand-foreground font-black text-xs flex items-center gap-1 transition-all cursor-pointer"
							onclick={handleSaveNick}
						>
							<Check class="w-3.5 h-3.5 stroke-[3]" />
							Salvar
						</button>
						<button 
							type="button"
							class="h-9 px-3 rounded-full bg-fg/5 hover:bg-fg/10 text-fg/60 text-xs flex items-center justify-center transition-all cursor-pointer"
							onclick={() => { isEditingNick = false; editedNick = currentUsername; }}
						>
							<X class="w-3.5 h-3.5" />
						</button>
					</div>
				{:else}
					<div class="flex items-center gap-2">
						<h2 class="text-xl font-black text-fg tracking-tight">{currentUsername}</h2>
						<button 
							type="button" 
							class="p-2 rounded-full hover:bg-fg/10 text-fg/40 hover:text-fg transition-all cursor-pointer"
							title="Mudar Nickname"
							onclick={() => { editedNick = currentUsername; isEditingNick = true; }}
						>
							<Pencil class="w-3.5 h-3.5" />
						</button>
					</div>
				{/if}

				<div class="flex items-center gap-2 pt-1">
					{#if isMicrosoft}
						<span class="bg-emerald-500/15 text-emerald-400 border border-emerald-500/30 text-[10px] font-bold px-2.5 py-0.5 rounded-full flex items-center gap-1">
							<ShieldCheck class="w-3 h-3" /> Conta Oficial Microsoft
						</span>
					{:else}
						<span class="bg-amber-500/15 text-amber-300 border border-amber-500/30 text-[10px] font-bold px-2.5 py-0.5 rounded-full flex items-center gap-1">
							<Gamepad2 class="w-3 h-3" /> Modo Offline / Pirata
						</span>
					{/if}
					<span class="text-fg/30 text-[10px] font-mono truncate max-w-[170px]" title={account.value?.uuid}>
						UUID: {account.value?.uuid?.slice(0, 10)}...
					</span>
				</div>
			</div>

			<!-- Player Stats Cards -->
			<div class="grid grid-cols-2 gap-2.5 mt-5">
				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col justify-between">
					<span class="text-[10px] font-bold text-fg/40 uppercase flex items-center gap-1">
						<Clock class="w-3 h-3 text-brand-500" /> Tempo Total
					</span>
					<div class="text-base font-black text-fg mt-1">{gamingStats.formattedTotalTime}</div>
					<span class="text-[9px] text-fg/30 mt-0.5">Tempo acumulado</span>
				</div>

				<div class="bg-bg-subtle border border-fg/5 rounded-2xl p-3.5 flex flex-col justify-between">
					<span class="text-[10px] font-bold text-fg/40 uppercase flex items-center gap-1">
						<Layers class="w-3 h-3 text-emerald-400" /> Sessão Atual
					</span>
					<div class="text-base font-black text-fg mt-1">{gamingStats.formattedTodayTime}</div>
					<span class="text-[9px] text-fg/30 mt-0.5">Tempo jogado hoje</span>
				</div>
			</div>

			<!-- Actions Section -->
			<div class="mt-6 pt-5 border-t border-fg/5 flex flex-col gap-2">
				<button 
					type="button"
					class="w-full h-11 rounded-full bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 hover:border-red-500/40 text-red-400 font-bold text-xs flex items-center justify-center gap-2 transition-all active:scale-98 cursor-pointer"
					onclick={handleLogout}
				>
					<LogOut class="w-4 h-4" />
					Sair da Conta (Logout)
				</button>
			</div>
		</div>
	</div>
</div>
