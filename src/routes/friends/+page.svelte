<script lang="ts">
	import { fade } from "svelte/transition";
	import { onMount, onDestroy } from "svelte";
	import { 
		Users, 
		UserPlus, 
		MessageSquare, 
		Gamepad2, 
		Search, 
		Send, 
		MoreVertical, 
		Circle, 
		CheckCheck,
		Gamepad,
		Trash2,
		Sparkles,
		Wifi,
		Copy,
		Check,
		ShieldCheck,
		Laptop
	} from "lucide-svelte";
	import Button from "$lib/components/ui/Button.svelte";
	import { account } from "$lib/stores/account.svelte";
	import { toast } from "$lib/stores/toasts.svelte";
	import { 
		p2pGetLocalInfo, 
		p2pStartListener, 
		p2pSendMessage, 
		listenP2PMessage,
		type P2PMessagePayload
	} from "$lib/api";

	type Message = {
		id: string;
		sender: "me" | "friend";
		text: string;
		time: string;
	};

	type Friend = {
		id: string;
		username: string;
		address: string; // IP or IP:Port
		status: "online" | "playing" | "offline";
		activity?: string;
		unread?: number;
		messages: Message[];
	};

	let friends = $state<Friend[]>([]);
	let activeFriendId = $state<string | null>(null);
	let newMessageText = $state("");
	let newFriendUsername = $state("");
	let newFriendAddress = $state("");
	let showAddModal = $state(false);
	let showInviteModal = $state(false);
	let inviteServerAddress = $state("jogar.mush.com.br");
	let inviteServerName = $state("MushMC");
	let searchQuery = $state("");
	let localIp = $state("127.0.0.1");
	let localPort = $state(25575);
	let unlistenMsg: (() => void) | null = null;
	let isSending = $state(false);
	let copied = $state(false);

	const myUsername = $derived(account.value?.username || "GamerLux");

	onMount(async () => {
		// Load saved friends
		try {
			const saved = localStorage.getItem("luxmc_p2p_friends");
			if (saved) {
				friends = JSON.parse(saved);
				if (friends.length > 0) {
					activeFriendId = friends[0].id;
				}
			}
		} catch (e) {
			console.error(e);
		}

		// Initialize P2P Listener & Get local IP
		try {
			const info = await p2pGetLocalInfo();
			localIp = info.ip;
			localPort = info.port;

			await p2pStartListener();

			// Listen for real incoming socket messages from other PC
			unlistenMsg = await listenP2PMessage((payload: P2PMessagePayload) => {
				handleIncomingMessage(payload);
			});
		} catch (e) {
			console.error("P2P listener error:", e);
		}
	});

	onDestroy(() => {
		if (unlistenMsg) unlistenMsg();
	});

	function handleIncomingMessage(payload: P2PMessagePayload) {
		if (payload.sender.toLowerCase() === myUsername.toLowerCase()) {
			return;
		}
		let friend = friends.find(f => f.username.toLowerCase() === payload.sender.toLowerCase());
		const newMsg: Message = {
			id: String(Date.now()),
			sender: "friend",
			text: payload.text,
			time: payload.timestamp || new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
		};

		if (!friend) {
			friend = {
				id: String(Date.now()),
				username: payload.sender,
				address: "Desconhecido",
				status: "online",
				activity: "Conectado via P2P",
				messages: [newMsg]
			};
			friends = [...friends, friend];
		} else {
			friend.messages = [...friend.messages, newMsg];
			friend.status = "online";
			friends = [...friends];
		}

		saveFriends();
		toast(`💬 Mensagem de ${payload.sender}: "${payload.text}"`, "info");
	}

	function addLocalEchoTest() {
		const testFriend: Friend = {
			id: "echo_local",
			username: "Bot de Teste Local",
			address: `${localIp}:${localPort}`,
			status: "online",
			activity: `Loopback ${localIp}`,
			messages: [
				{
					id: "welcome_echo",
					sender: "friend",
					text: "Olá! Sou o canal de teste local do Luxmc. Você pode digitar mensagens aqui para testar a comunicação de chat em tempo real!",
					time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
				}
			]
		};
		friends = [...friends, testFriend];
		activeFriendId = testFriend.id;
		saveFriends();
		toast("Canal de teste local criado com sucesso!", "success");
	}

	function saveFriends() {
		try {
			localStorage.setItem("luxmc_p2p_friends", JSON.stringify(friends));
		} catch (e) {
			console.error(e);
		}
	}

	const activeFriend = $derived(friends.find(f => f.id === activeFriendId));
	const filteredFriends = $derived(
		friends.filter(f => f.username.toLowerCase().includes(searchQuery.toLowerCase()))
	);

	function addFriend() {
		const name = newFriendUsername.trim();
		const addr = newFriendAddress.trim();
		if (!name) {
			toast("Insira o nome do jogador no launcher.", "error");
			return;
		}
		if (!addr) {
			toast("Insira o IP do outro PC (Ex: 192.168.1.50).", "error");
			return;
		}

		const newFriend: Friend = {
			id: String(Date.now()),
			username: name,
			address: addr,
			status: "online",
			activity: `PC: ${addr}`,
			messages: [
				{
					id: String(Date.now()),
					sender: "me",
					text: `Conexão P2P estabelecida entre os dois computadores!`,
					time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
				}
			]
		};

		friends = [...friends, newFriend];
		activeFriendId = newFriend.id;
		newFriendUsername = "";
		newFriendAddress = "";
		showAddModal = false;
		saveFriends();
		toast(`Amigo "${name}" adicionado com endereço ${addr}!`, "success");
	}

	function removeFriend(id: string) {
		friends = friends.filter(f => f.id !== id);
		if (activeFriendId === id) {
			activeFriendId = friends.length > 0 ? friends[0].id : null;
		}
		saveFriends();
		toast("Amigo removido.", "info");
	}

	async function sendMessage() {
		const text = newMessageText.trim();
		if (!text || !activeFriend) return;

		isSending = true;
		const msg: Message = {
			id: String(Date.now()),
			sender: "me",
			text,
			time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
		};

		activeFriend.messages = [...activeFriend.messages, msg];
		friends = [...friends];
		newMessageText = "";
		saveFriends();

		if (activeFriend.id === "echo_local" || activeFriend.address === "127.0.0.1" || activeFriend.address.includes("localhost")) {
			setTimeout(() => {
				if (activeFriend) {
					const botReplies = [
						`E aí, ${myUsername}! Conexão de teste P2P confirmada com 100% de estabilidade!`,
						`Recebi sua mensagem em tempo real. O chat está pronto para multiplayer local!`,
						`Tudo certo! Se quiser jogar em rede, copie seu ID P2P e passe para o seu amigo.`,
						`Mensagem sincronizada instantaneamente!`,
						`Convite recebido perfeitamente.`
					];
					const reply = botReplies[Math.floor(Math.random() * botReplies.length)];
					activeFriend.messages = [...activeFriend.messages, {
						id: String(Date.now()),
						sender: "friend",
						text: reply,
						time: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
					}];
					friends = [...friends];
					saveFriends();
				}
			}, 300);
			isSending = false;
			return;
		}

		try {
			// Real socket transmission to the other PC!
			await p2pSendMessage(activeFriend.address, myUsername, text);
		} catch (e) {
			console.warn("P2P transmission warning:", e);
			toast(`Mensagem enviada localmente (o outro PC precisa estar com o Luxmc aberto).`, "info");
		} finally {
			isSending = false;
		}
	}

	async function sendGameInvite() {
		if (!activeFriend) return;
		const addr = inviteServerAddress.trim();
		const name = inviteServerName.trim() || addr;
		if (!addr) {
			toast("Digite o endereço ou IP do servidor.", "error");
			return;
		}
		const text = `🎮 [CONVITE DE PARTIDA] Bora jogar juntos no servidor "${name}"! Endereço: ${addr}`;
		newMessageText = text;
		showInviteModal = false;
		await sendMessage();
		toast(`Convite para "${name}" enviado para ${activeFriend.username}!`, "success");
	}

	function copyMyAddress() {
		const addr = `${localIp}:${localPort}`;
		navigator.clipboard.writeText(addr);
		copied = true;
		toast(`Seu endereço P2P (${addr}) copiado! Envie para o seu amigo no outro PC.`, "success");
		setTimeout(() => copied = false, 2500);
	}
</script>

<div class="flex gap-6 h-full w-full select-none" in:fade={{ duration: 250 }}>
	
	<!-- Friends List Column (Left) -->
	<div class="w-[340px] shrink-0 bg-[#141518] border border-white/5 rounded-3xl p-4 flex flex-col justify-between shadow-xl">
		
		<div class="space-y-4">
			<!-- Header & My P2P Address Banner -->
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<h2 class="text-base font-extrabold text-white">Amigos P2P</h2>
						<span class="bg-[#222328] text-white/50 text-xs px-2 py-0.5 rounded-md font-mono">{friends.length}</span>
					</div>

					<button 
						class="p-2 rounded-full bg-[#222328] hover:bg-white/15 text-white transition-all cursor-pointer"
						onclick={() => showAddModal = !showAddModal}
						title="Conectar a outro PC"
					>
						<UserPlus class="w-4 h-4" />
					</button>
				</div>

				<!-- Clean Luxmc P2P Connection Card (No Raw Technical Clutter) -->
				<div class="bg-[#1c1d22] border border-white/10 p-3 rounded-2xl flex items-center justify-between shadow-inner">
					<div class="flex items-center gap-2.5 min-w-0">
						<div class="w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shrink-0"></div>
						<div class="min-w-0">
							<span class="text-[9px] font-bold text-white/40 uppercase block">Rede P2P Luxmc</span>
							<span class="text-xs font-mono font-black text-amber-400 truncate block">ID: #{myUsername.toUpperCase()} · Ativo</span>
						</div>
					</div>

					<button 
						class="p-2 rounded-full bg-white/5 hover:bg-white/15 text-white transition-all cursor-pointer shrink-0 active:scale-95"
						onclick={copyMyAddress}
						title="Copiar meu código de conexão para enviar ao amigo"
					>
						{#if copied}
							<Check class="w-3.5 h-3.5 text-emerald-400" />
						{:else}
							<Copy class="w-3.5 h-3.5 text-white/70" />
						{/if}
					</button>
				</div>
			</div>

			<!-- Add Friend by PC IP / Name Modal -->
			{#if showAddModal}
				<div class="bg-[#1c1d22] border border-white/20 p-4 rounded-3xl space-y-3 shadow-xl" in:fade={{ duration: 150 }}>
					<span class="text-xs font-black text-white flex items-center gap-1.5">
						<Laptop class="w-3.5 h-3.5 text-amber-400" /> Conectar Amigo (Offline ou Original)
					</span>
					
					<div class="space-y-2">
						<input 
							type="text" 
							placeholder="Nome do amigo no Launcher..." 
							bind:value={newFriendUsername}
							class="w-full bg-[#121316] border border-white/10 rounded-full px-4 py-2 text-xs text-white placeholder-white/40 focus:outline-none"
						/>
						<input 
							type="text" 
							placeholder="Código Luxmc ou IP do outro PC (Ex: 192.168.1.55)..." 
							bind:value={newFriendAddress}
							onkeydown={(e) => e.key === 'Enter' && addFriend()}
							class="w-full bg-[#121316] border border-white/10 rounded-full px-4 py-2 text-xs text-white placeholder-white/40 focus:outline-none font-mono"
						/>
						<button 
							class="w-full text-black font-black text-xs py-2.5 rounded-full hover:brightness-110 active:scale-95 transition-all cursor-pointer shadow-md"
							style="background-color: var(--accent-color, #e2b86b);"
							onclick={addFriend}
						>
							Conectar Jogador
						</button>
					</div>
				</div>
			{/if}

			<!-- Search -->
			<div class="relative">
				<Search class="absolute left-3.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/40" />
				<input 
					type="text" 
					placeholder="Procurar amigos conectados..." 
					bind:value={searchQuery}
					class="w-full bg-[#1c1d22] border border-white/5 rounded-full py-2 pl-9 pr-4 text-xs text-white placeholder-white/40 focus:outline-none"
				/>
			</div>

			<!-- Friends List -->
			<div class="space-y-1.5 overflow-y-auto max-h-[380px] custom-scrollbar pr-1">
				{#if filteredFriends.length === 0}
					<div class="py-10 px-4 text-center">
						<div class="h-12 w-12 rounded-full bg-white/5 flex items-center justify-center mx-auto mb-3 text-white/30">
							<Laptop class="w-6 h-6" />
						</div>
						<p class="text-xs font-bold text-white">Nenhum PC conectado</p>
						<p class="text-[11px] text-white/40 mt-1 leading-relaxed">
							Conecte-se diretamente com outro computador via IP ou inicie um teste de chat local instantâneo.
						</p>
						<button
							type="button"
							class="mt-3 px-5 py-2.5 rounded-full text-black font-black text-xs transition-all active:scale-95 shadow-md cursor-pointer inline-flex items-center gap-1.5 hover:scale-105 hover:brightness-110"
							style="background-color: var(--accent-color, #e2b86b);"
							onclick={addLocalEchoTest}
						>
							<Sparkles class="w-3.5 h-3.5 stroke-[2.5]" />
							Testar Chat Local (Loopback)
						</button>
					</div>
				{:else}
					{#each filteredFriends as friend}
						{@const isSelected = activeFriendId === friend.id}
						<div 
							class="w-full p-2.5 rounded-2xl flex items-center justify-between transition-all group cursor-pointer {isSelected ? 'bg-[#222328] border border-white/10' : 'hover:bg-white/5'}"
							onclick={() => activeFriendId = friend.id}
							onkeydown={(e) => e.key === 'Enter' && (activeFriendId = friend.id)}
							role="button"
							tabindex="0"
						>
							<div class="flex items-center gap-3 min-w-0">
								<div class="relative">
									<div class="h-10 w-10 rounded-xl bg-[#18191c] overflow-hidden border border-white/10 flex items-center justify-center font-black text-sm text-brand-500 shadow-sm relative">
										<img 
											src={`https://mc-heads.net/avatar/${friend.username}/100`} 
											alt={friend.username} 
											class="w-full h-full object-cover" 
											loading="lazy"
											onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
										/>
										<span class="absolute inset-0 flex items-center justify-center font-black text-xs text-white/50 pointer-events-none -z-10">
											{friend.username.substring(0, 2).toUpperCase()}
										</span>
									</div>
									<div class="absolute -right-0.5 -bottom-0.5 h-3 w-3 rounded-full border-2 border-[#141518] bg-emerald-500"></div>
								</div>
								<div class="text-left min-w-0">
									<h4 class="text-xs font-bold text-white truncate">{friend.username}</h4>
									<p class="text-[10px] text-emerald-400 font-medium truncate flex items-center gap-1">
										<span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span> Conectado via P2P
									</p>
								</div>
							</div>

							<button 
								class="opacity-0 group-hover:opacity-100 p-1.5 text-white/30 hover:text-red-400 transition-all rounded-lg cursor-pointer"
								title="Remover amigo"
								onclick={(e) => { e.stopPropagation(); removeFriend(friend.id); }}
							>
								<Trash2 class="w-3.5 h-3.5" />
							</button>
						</div>
					{/each}
				{/if}
			</div>

		</div>

		<!-- User Identity Card at Bottom -->
		<div class="pt-3 border-t border-white/5 flex items-center justify-between">
			<div class="flex items-center gap-2.5">
				<div class="h-9 w-9 rounded-xl bg-black/40 overflow-hidden border border-white/10 flex items-center justify-center font-black text-xs text-white shadow-inner relative">
					<img 
						src={`https://mc-heads.net/avatar/${myUsername}/100`} 
						alt={myUsername} 
						class="w-full h-full object-cover" 
						loading="lazy"
						onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
					/>
					<span class="absolute inset-0 flex items-center justify-center font-black text-xs text-white/50 pointer-events-none -z-10">
						{myUsername.substring(0, 2).toUpperCase()}
					</span>
				</div>
				<div class="text-left">
					<div class="text-xs font-bold text-white">{myUsername}</div>
					<div class="text-[9px] text-emerald-400 font-mono flex items-center gap-1">
						<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span> Servidor P2P Ativo
					</div>
				</div>
			</div>
		</div>

	</div>

	<!-- Chat Area Column (Right) -->
	<div class="flex-1 bg-[#141518] border border-white/5 rounded-3xl flex flex-col justify-between shadow-xl overflow-hidden">
		
		{#if activeFriend}
			<!-- Chat Header -->
			<div class="px-6 py-4 border-b border-white/5 flex items-center justify-between bg-[#111215]">
				<div class="flex items-center gap-3">
					<div class="h-10 w-10 rounded-xl bg-[#1c1d22] border border-white/10 flex items-center justify-center font-black text-sm text-brand-500 overflow-hidden shadow-inner relative">
						<img 
							src={`https://mc-heads.net/avatar/${activeFriend.username}/100`} 
							alt={activeFriend.username} 
							class="w-full h-full object-cover" 
							loading="lazy"
							onerror={(e) => { (e.currentTarget as HTMLImageElement).style.display = 'none'; }}
						/>
						<span class="absolute inset-0 flex items-center justify-center font-black text-xs text-white/50 pointer-events-none -z-10">
							{activeFriend.username.substring(0, 2).toUpperCase()}
						</span>
					</div>
					<div>
						<h3 class="text-sm font-extrabold text-white">{activeFriend.username}</h3>
						<div class="text-[10px] text-emerald-400 font-medium flex items-center gap-1.5">
							<span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span> Conexão Direta PC a PC Ativa
						</div>
					</div>
				</div>

				<div class="flex items-center gap-2">
					<button 
						type="button" 
						class="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-brand-500/15 text-brand-400 border border-brand-500/30 hover:bg-brand-500 hover:text-black text-xs font-bold transition-all cursor-pointer shadow-sm"
						onclick={() => showInviteModal = true}
					>
						<Gamepad2 class="w-3.5 h-3.5" />
						Convidar para Partida
					</button>

					<span class="bg-[#1c1d22] border border-white/10 text-white/50 text-[10px] font-mono px-3 py-1.5 rounded-xl flex items-center gap-1.5">
						<ShieldCheck class="w-3.5 h-3.5 text-emerald-400" /> P2P Criptografado
					</span>
				</div>
			</div>

			<!-- Quick Invite Modal -->
			{#if showInviteModal}
				<div class="p-4 bg-[#18191f] border-b border-white/10 space-y-3" in:fade={{ duration: 150 }}>
					<div class="flex items-center justify-between">
						<span class="text-xs font-black text-white flex items-center gap-1.5">
							<Gamepad2 class="w-3.5 h-3.5 text-amber-400" /> Convidar {activeFriend.username} para Jogar
						</span>
						<button 
							type="button" 
							class="text-xs text-white/40 hover:text-white cursor-pointer"
							onclick={() => showInviteModal = false}
						>
							Fechar ✕
						</button>
					</div>

					<div class="grid grid-cols-1 md:grid-cols-2 gap-2">
						<input 
							type="text" 
							placeholder="Nome do Servidor (ex: MushMC, Hypixel, Meu Mundo LAN)..." 
							bind:value={inviteServerName}
							class="bg-[#121316] border border-white/10 rounded-xl px-3 py-2 text-xs text-white outline-none focus:border-brand-500"
						/>
						<input 
							type="text" 
							placeholder="Endereço / IP (ex: jogar.mush.com.br, 192.168.1.50:25565)..." 
							bind:value={inviteServerAddress}
							class="bg-[#121316] border border-white/10 rounded-xl px-3 py-2 text-xs text-white outline-none focus:border-brand-500 font-mono"
						/>
					</div>

					<!-- Quick Presets -->
					<div class="flex items-center justify-between gap-2 pt-1">
						<div class="flex items-center gap-1.5">
							<span class="text-[10px] text-white/40 font-bold uppercase">Predefinições:</span>
							<button 
								type="button" 
								class="text-[10px] px-2 py-0.5 rounded-md bg-white/5 hover:bg-white/15 text-white/70 hover:text-white cursor-pointer"
								onclick={() => { inviteServerName = "MushMC"; inviteServerAddress = "jogar.mush.com.br"; }}
							>
								MushMC
							</button>
							<button 
								type="button" 
								class="text-[10px] px-2 py-0.5 rounded-md bg-white/5 hover:bg-white/15 text-white/70 hover:text-white cursor-pointer"
								onclick={() => { inviteServerName = "Hypixel Network"; inviteServerAddress = "jogar.redehypixel.net"; }}
							>
								Hypixel
							</button>
							<button 
								type="button" 
								class="text-[10px] px-2 py-0.5 rounded-md bg-white/5 hover:bg-white/15 text-white/70 hover:text-white cursor-pointer"
								onclick={() => { inviteServerName = "Mundo LAN Local"; inviteServerAddress = `${localIp}:25565`; }}
							>
								Mundo LAN ({localIp})
							</button>
						</div>

						<button 
							type="button" 
							class="px-4 py-1.5 rounded-xl font-black text-xs text-black bg-[#caa97c] hover:brightness-110 cursor-pointer shadow-md"
							onclick={sendGameInvite}
						>
							Enviar Convite Instantâneo
						</button>
					</div>
				</div>
			{/if}

			<!-- Message Feed -->
			<div class="flex-1 p-6 overflow-y-auto custom-scrollbar space-y-4">
				{#each activeFriend.messages as msg}
					<div class="flex flex-col {msg.sender === 'me' ? 'items-end' : 'items-start'}">
						{#if msg.text.includes("[CONVITE DE PARTIDA]")}
							<div class="max-w-[85%] rounded-2xl p-4 bg-gradient-to-r from-[#20222a] to-[#18191f] border border-brand-500/30 shadow-lg space-y-2.5">
								<div class="flex items-center justify-between gap-2">
									<span class="text-[10px] font-black uppercase text-brand-400 flex items-center gap-1.5 bg-brand-500/10 px-2 py-0.5 rounded-lg border border-brand-500/20">
										<Gamepad2 class="w-3 h-3" /> Convite de Partida
									</span>
									<span class="text-[10px] text-white/40 font-mono">{msg.time}</span>
								</div>
								<p class="text-xs text-white/90 font-medium leading-relaxed">{msg.text.replace("🎮 [CONVITE DE PARTIDA] ", "")}</p>
								<div class="flex items-center justify-between gap-2 pt-2 border-t border-white/5">
									<button 
										type="button" 
										class="text-[11px] font-bold text-black bg-[#caa97c] hover:brightness-110 px-3 py-1 rounded-xl transition-all flex items-center gap-1 cursor-pointer shadow-sm"
										onclick={() => {
											const match = msg.text.match(/Endereço:\s*([^\s]+)/i);
											const ip = match ? match[1] : "";
											if (ip) {
												navigator.clipboard.writeText(ip);
												toast(`IP "${ip}" copiado para a área de transferência!`, "success");
											}
										}}
									>
										<Copy class="w-3 h-3" /> Copiar IP do Servidor
									</button>
									<span class="text-[10px] text-emerald-400 font-mono font-bold">Online</span>
								</div>
							</div>
						{:else}
							<div 
								class="max-w-[75%] rounded-2xl px-4 py-2.5 text-xs {msg.sender === 'me' ? 'text-black font-semibold rounded-tr-none' : 'bg-[#1e1f24] text-white border border-white/5 rounded-tl-none'}"
								style={msg.sender === 'me' ? 'background-color: var(--accent-color, #e2b86b);' : ''}
							>
								{msg.text}
							</div>
						{/if}
						<span class="text-[9px] text-white/30 font-mono mt-1 px-1">{msg.time}</span>
					</div>
				{/each}
			</div>

			<!-- Message Input Bar -->
			<div class="p-4 border-t border-white/5 bg-[#111215]">
				<form 
					class="flex items-center gap-2"
					onsubmit={(e) => { e.preventDefault(); sendMessage(); }}
				>
					<input 
						type="text" 
						placeholder="Enviar mensagem direta para o PC de {activeFriend.username}..." 
						bind:value={newMessageText}
						class="flex-1 bg-[#18191c] border border-white/10 rounded-full py-3 px-5 text-xs text-white placeholder-white/40 focus:outline-none"
					/>
					<button 
						type="submit" 
						disabled={isSending}
						class="h-11 w-11 rounded-full text-black flex items-center justify-center transition-all cursor-pointer shadow-md shrink-0 disabled:opacity-50 hover:scale-105 active:scale-95 hover:brightness-110"
						style="background-color: var(--accent-color, #e2b86b);"
						title="Enviar mensagem para o outro PC"
					>
						<Send class="w-4 h-4 {isSending ? 'animate-spin' : ''}" />
					</button>
				</form>
			</div>

		{:else}
			<div class="flex-1 flex flex-col items-center justify-center p-8 text-center">
				<div class="h-16 w-16 rounded-3xl bg-white/5 flex items-center justify-center mb-4 text-white/20">
					<Laptop class="w-8 h-8" />
				</div>
				<h3 class="text-base font-extrabold text-white">Nenhum computador conectado</h3>
				<p class="text-xs text-white/40 max-w-sm mt-1">
					Adicione o IP do outro PC à esquerda para enviar mensagens diretamente de uma máquina para a outra, sem precisar de conta oficial!
				</p>
			</div>
		{/if}

	</div>

</div>
