<script lang="ts">
    import LuxAccountForm from "./LuxAccountForm.svelte";
	import { fade, scale } from "svelte/transition";
	import { 
		User, 
		Check, 
		X, 
		Sparkles, 
		LogIn, 
		Gamepad2,
		ShieldCheck,
		AlertCircle
	} from "lucide-svelte";
	import { account } from "$lib/stores/account.svelte";
	import { authLogin, authOfflineLogin, type AuthAccount } from "$lib/api";
	import { toast } from "$lib/stores/toasts.svelte";

	let { 
		isOpen = $bindable(false), 
		onClose,
		onAccountAdded
	}: { 
		isOpen: boolean; 
		onClose?: () => void;
		onAccountAdded?: (acc: AuthAccount) => void;
	} = $props();

	let tab = $state<"luxmc" | "offline" | "microsoft">("luxmc");
	let offlineUsername = $state("");
	let isLoggingIn = $state(false);
	let errorMsg = $state<string | null>(null);

	async function handleOfflineLogin() {
		const trimmed = offlineUsername.trim();
		if (!trimmed) {
			errorMsg = "Por favor, digite um nickname.";
			return;
		}
		if (trimmed.length < 3 || trimmed.length > 16) {
			errorMsg = "O nickname deve ter entre 3 e 16 caracteres.";
			return;
		}
		if (!/^[a-zA-Z0-9_]+$/.test(trimmed)) {
			errorMsg = "O nickname pode conter apenas letras, números e underline (_).";
			return;
		}

		isLoggingIn = true;
		errorMsg = null;
		try {
			const acc = await authOfflineLogin(trimmed);
			account.value = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(account.value));
			toast(`Conta offline "${trimmed}" adicionada com sucesso!`, "success");
			onAccountAdded?.(acc);
			handleClose();
		} catch (e) {
			errorMsg = "Falha ao entrar offline: " + String(e);
			toast(errorMsg, "error");
		} finally {
			isLoggingIn = false;
		}
	}

	async function handleMicrosoftLogin() {
		isLoggingIn = true;
		errorMsg = null;
		try {
			const acc = await authLogin();
			account.value = {
				id: acc.id,
				username: acc.username,
				uuid: acc.uuid,
				minecraftToken: acc.accessToken,
				expiresAt: acc.expiresAt
			};
			localStorage.setItem("luxmc_current_account", JSON.stringify(account.value));
			toast(`Conta Microsoft de ${acc.username} conectada!`, "success");
			onAccountAdded?.(acc as AuthAccount);
			handleClose();
		} catch (e) {
			errorMsg = "Falha no login Microsoft: " + String(e);
			toast(errorMsg, "error");
		} finally {
			isLoggingIn = false;
		}
	}

	function handleClose() {
		offlineUsername = "";
		errorMsg = null;
		isLoggingIn = false;
		isOpen = false;
		onClose?.();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") handleClose();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div 
		class="fixed inset-0 z-[10001] flex items-center justify-center bg-bg-overlay/75 backdrop-blur-md p-4 select-none"
		in:fade={{ duration: 150 }}
		out:fade={{ duration: 120 }}
		onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
	>
		<div 
			class="w-full max-w-md rounded-3xl bg-bg-elevated border border-fg/10 shadow-2xl overflow-hidden flex flex-col"
			in:scale={{ start: 0.95, duration: 180 }}
			out:scale={{ start: 0.95, duration: 120 }}
		>
			<!-- Header -->
			<div class="p-5 border-b border-fg/5 bg-bg-elevated flex items-center justify-between">
				<div class="flex items-center gap-2.5">
					<div class="w-8 h-8 rounded-xl bg-brand-500/10 border border-brand-500/30 flex items-center justify-center text-brand-500">
						<Gamepad2 class="w-4 h-4" />
					</div>
					<div>
						<h3 class="text-sm font-black text-fg">Adicionar Conta</h3>
						<p class="text-[10px] text-fg/40">Selecione o tipo de conta para jogar</p>
					</div>
				</div>
				<button 
					type="button" 
					class="w-7 h-7 rounded-xl bg-fg/5 hover:bg-fg/10 text-fg/50 hover:text-fg flex items-center justify-center transition-colors cursor-pointer text-xs"
					onclick={handleClose}
				>
					✕
				</button>
			</div>

			<!-- Tab Switcher (Offline vs Microsoft) -->
			<div class="p-4">
				<div class="grid grid-cols-3 gap-2 p-1 bg-bg-elevated rounded-2xl border border-fg/5">
                    <button type="button" class="rounded-xl px-3 py-2 text-xs font-bold {tab === 'luxmc' ? 'bg-brand-500 text-brand-foreground' : 'text-fg-muted'}" onclick={() => tab = "luxmc"}>Luxmc</button>
					<button 
						type="button"
						class="py-2 px-3 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-2 {tab === 'offline' ? 'bg-brand-500 text-brand-foreground shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
						onclick={() => { tab = 'offline'; errorMsg = null; }}
					>
						<User class="w-3.5 h-3.5" />
						Conta Offline
					</button>
					<button 
						type="button"
						class="py-2 px-3 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center gap-2 {tab === 'microsoft' ? 'bg-emerald-500 text-brand-foreground shadow-md' : 'text-fg/60 hover:text-fg hover:bg-fg/5'}"
						onclick={() => { tab = 'microsoft'; errorMsg = null; }}
					>
						<ShieldCheck class="w-3.5 h-3.5" />
						Microsoft
					</button>
				</div>

				<!-- Offline Tab Body -->
				{#if tab === 'luxmc'}
                    <LuxAccountForm onAuthenticated={(value) => { onAccountAdded?.(value); handleClose(); }} />
                {:else if tab === 'offline'}
					<form onsubmit={(e) => { e.preventDefault(); handleOfflineLogin(); }} class="space-y-4 pt-4">
						<div class="space-y-1.5">
							<label for="offline-nick" class="text-xs font-bold text-fg/70 block">Nickname (Nome de Jogador)</label>
							<div class="relative">
								<input 
									id="offline-nick"
									type="text"
									bind:value={offlineUsername}
									placeholder="ex: Steve_123, ProMiner"
									maxlength="16"
									class="w-full h-11 px-4 rounded-xl bg-bg-subtle border border-fg/10 text-xs font-bold text-fg outline-none focus:border-brand-500 transition-all placeholder:text-fg/30"
								/>
							</div>
							<p class="text-[10px] text-fg/40">Perfil local, sem sincronização com o site.</p>
						</div>

						{#if errorMsg}
							<div class="p-3 bg-red-500/10 border border-red-500/25 rounded-xl text-xs text-red-400 font-bold flex items-center gap-2">
								<AlertCircle class="w-4 h-4 shrink-0" />
								<span>{errorMsg}</span>
							</div>
						{/if}

						<button 
							type="submit"
							disabled={isLoggingIn || !offlineUsername.trim()}
							class="w-full h-11 rounded-xl bg-brand-500 hover:bg-brand-400 disabled:opacity-50 text-brand-foreground font-black text-xs transition-all shadow-lg active:scale-98 flex items-center justify-center gap-2 cursor-pointer"
						>
							{#if isLoggingIn}
								<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
								Entrando...
							{:else}
								<Check class="w-4 h-4 stroke-[3]" />
								Entrar com Conta Offline
							{/if}
						</button>
					</form>
				{:else}
					<!-- Microsoft Tab Body -->
					<div class="space-y-4 pt-4 text-center">
						<div class="p-4 bg-emerald-500/10 border border-emerald-500/20 rounded-2xl space-y-1 text-left">
							<span class="text-xs font-bold text-emerald-400 block flex items-center gap-1.5">
								<ShieldCheck class="w-4 h-4" /> Autenticação Oficial Microsoft
							</span>
							<p class="text-[10px] text-fg/60 leading-relaxed">
								Acesse servidores oficiais da Mojang (Hypixel, Realms) e sincronize sua skin original com segurança.
							</p>
						</div>

						{#if errorMsg}
							<div class="p-3 bg-red-500/10 border border-red-500/25 rounded-xl text-xs text-red-400 font-bold flex items-center gap-2 text-left">
								<AlertCircle class="w-4 h-4 shrink-0" />
								<span>{errorMsg}</span>
							</div>
						{/if}

						<button 
							type="button"
							disabled={isLoggingIn}
							onclick={handleMicrosoftLogin}
							class="w-full h-11 rounded-xl bg-emerald-500 hover:bg-emerald-400 disabled:opacity-50 text-brand-foreground font-black text-xs transition-all shadow-lg active:scale-98 flex items-center justify-center gap-2 cursor-pointer"
						>
							{#if isLoggingIn}
								<div class="w-4 h-4 rounded-full border-2 border-bg-overlay border-t-transparent animate-spin"></div>
								Aguardando login no navegador...
							{:else}
								<LogIn class="w-4 h-4" />
								Entrar com a Microsoft
							{/if}
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
