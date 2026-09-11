<script lang="ts">
	import { X } from "lucide-svelte";
	import { toast } from "$lib/stores/toasts.svelte";

	type Props = {
		open: boolean;
		onclose: () => void;
	};

	let { open, onclose }: Props = $props();

	let serverName = $state("");
	let serverHost = $state("");
	let serverPort = $state("25565");
	let isTesting = $state(false);

	function resetForm() {
		serverName = "";
		serverHost = "";
		serverPort = "25565";
	}

	function handleClose() {
		resetForm();
		onclose();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) handleClose();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") handleClose();
	}

	async function testConnection() {
		if (!serverHost.trim()) {
			toast("Insira o endereço do servidor.", "error");
			return;
		}
		isTesting = true;
		try {
			await new Promise((r) => setTimeout(r, 1500));
			toast(`Conexão com ${serverHost}:${serverPort} verificada com sucesso!`, "success");
		} catch {
			toast("Falha ao conectar ao servidor.", "error");
		} finally {
			isTesting = false;
		}
	}

	function addServer() {
		if (!serverName.trim() || !serverHost.trim()) {
			toast("Preencha todos os campos obrigatórios.", "error");
			return;
		}
		toast(`Servidor "${serverName}" adicionado com sucesso!`, "success");
		handleClose();
	}
</script>

{#if open}
	<div
		class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center"
		onclick={handleBackdropClick}
		onkeydown={handleKeydown}
		role="dialog"
		tabindex="-1"
	>
		<div class="bg-[#18191c] border border-white/10 rounded-3xl p-6 w-full max-w-md shadow-2xl space-y-5">
			<div class="flex items-center justify-between">
				<h3 class="text-base font-extrabold text-white">Adicionar Servidor</h3>
				<button
					type="button"
					class="w-8 h-8 rounded-xl bg-white/5 hover:bg-white/10 text-white/50 hover:text-white flex items-center justify-center transition-colors cursor-pointer"
					onclick={handleClose}
				>
					<X class="w-4 h-4" />
				</button>
			</div>

			<div class="space-y-3">
				<div>
					<label for="server-name" class="text-xs font-bold text-white/60 block mb-1">Nome do Servidor</label>
					<input
						id="server-name"
						type="text"
						placeholder="Meu Servidor"
						bind:value={serverName}
						class="w-full bg-[#141518] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white placeholder:text-white/30 focus:outline-none focus:border-brand-500 transition-all"
					/>
				</div>

				<div class="grid grid-cols-3 gap-3">
					<div class="col-span-2">
						<label for="server-host" class="text-xs font-bold text-white/60 block mb-1">Endereço (IP)</label>
						<input
							id="server-host"
							type="text"
							placeholder="mc.exemplo.com"
							bind:value={serverHost}
							class="w-full bg-[#141518] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono placeholder:text-white/30 focus:outline-none focus:border-brand-500 transition-all"
						/>
					</div>
					<div>
						<label for="server-port" class="text-xs font-bold text-white/60 block mb-1">Porta</label>
						<input
							id="server-port"
							type="text"
							placeholder="25565"
							bind:value={serverPort}
							class="w-full bg-[#141518] border border-white/10 rounded-2xl px-4 py-2.5 text-xs text-white font-mono placeholder:text-white/30 focus:outline-none focus:border-brand-500 transition-all"
						/>
					</div>
				</div>
			</div>

			<div class="flex gap-3 pt-2">
				<button
					type="button"
					class="flex-1 bg-[#24252a] hover:bg-white/10 text-white text-xs font-bold py-2.5 rounded-2xl border border-white/10 transition-all cursor-pointer active:scale-95"
					disabled={isTesting}
					onclick={testConnection}
				>
					{isTesting ? "Testando..." : "Testar Conexão"}
				</button>
				<button
					type="button"
					class="flex-1 bg-brand-500 hover:bg-[#ebd095] text-black text-xs font-black py-2.5 rounded-2xl transition-all cursor-pointer active:scale-95 shadow-md"
					onclick={addServer}
				>
					Adicionar
				</button>
			</div>
		</div>
	</div>
{/if}
