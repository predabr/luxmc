<script lang="ts">
    import { ArrowLeft, ArrowRight, LockKeyhole, Cloud, Loader2 } from "lucide-svelte";
    import { luxAccountLogin } from "$lib/api/luxAccount";
    import { openPortalAccount } from "$lib/api/system";
    import { account } from "$lib/stores/account.svelte";
    import type { AuthAccount } from "$lib/api/auth";
    import { activeSkinStore } from "$lib/stores/skin.svelte";
    import Button from "$lib/components/ui/Button.svelte";
    let { onAuthenticated }: { onAuthenticated?: (value: AuthAccount) => void } = $props();
    let username = $state("");
    let password = $state("");
    let step = $state<"nickname" | "password">("nickname");
    let error = $state("");
    let busy = $state(false);
    async function submit(event: SubmitEvent) {
        event.preventDefault();
        error = "";
        if (step === "nickname") {
            if (!/^[A-Za-z0-9_]{3,16}$/.test(username.trim())) { error = "Use de 3 a 16 letras, números ou _."; return; }
            username = username.trim(); step = "password"; return;
        }
        busy = true;
        try {
            const result = await luxAccountLogin(username, password);
            password = "";
            account.value = { id: result.id, uuid: result.uuid, username: result.username, minecraftToken: "", expiresAt: 0, skinUrl: result.skinUrl, skinVariant: result.skinVariant };
            localStorage.setItem("luxmc_current_account", JSON.stringify(account.value));
            activeSkinStore.setSkin({ id: result.uuid, name: result.username, url: `https://mc-heads.net/body/${result.username}/300`, skinUrl: result.skinUrl || `https://minotar.net/skin/${result.username}`, avatarUrl: `https://mc-heads.net/avatar/${result.username}/100`, type: "steve" });
            onAuthenticated?.(result);
        } catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
</script>
<form onsubmit={submit} class="space-y-4 py-4">
    <div class="flex items-center gap-3 rounded-2xl border border-brand-500/20 bg-brand-500/5 p-4">
        <Cloud class="h-6 w-6 text-brand-400" />
        <div><h2 class="text-sm font-bold text-fg">Sua conta Luxmc</h2><p class="mt-1 text-xs text-fg-muted">O mesmo nickname, amigos e preferências do site.</p></div>
    </div>
    {#if step === "nickname"}
        <label for="lux-nickname" class="block text-xs font-semibold text-fg-muted">Nickname cadastrado no site</label>
        <input id="lux-nickname" bind:value={username} required minlength="3" maxlength="16" autocomplete="username" placeholder="pedrin1234" class="w-full rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm text-fg focus:border-brand-400 focus:outline-none" />
    {:else}
        <div class="flex items-center gap-3"><img src={`https://mc-heads.net/avatar/${username}/64`} alt="" class="h-10 w-10 rounded-xl" /><span class="text-sm font-bold">{username}</span><button type="button" class="ml-auto text-xs text-fg-muted hover:text-fg" disabled={busy} onclick={() => { step = "nickname"; password = ""; error = ""; }}><ArrowLeft class="inline h-3 w-3" /> Trocar</button></div>
        <input type="text" name="username" value={username} autocomplete="username" class="sr-only" tabindex="-1" aria-label="Nickname" readonly />
        <label for="lux-password" class="block text-xs font-semibold text-fg-muted">Senha da conta Luxmc</label>
        <input id="lux-password" bind:value={password} type="password" required minlength="8" maxlength="128" autocomplete="current-password" placeholder="Sua senha" class="w-full rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm text-fg focus:border-brand-400 focus:outline-none" />
    {/if}
    {#if error}<p role="alert" class="rounded-xl border border-danger/20 bg-danger/10 p-3 text-xs text-danger">{error}</p>{/if}
    <Button type="submit" variant="primary" block loading={busy}>{#if busy}<Loader2 class="h-4 w-4 animate-spin" />{:else if step === "password"}<LockKeyhole class="h-4 w-4" />{:else}<ArrowRight class="h-4 w-4" />{/if}{step === "nickname" ? "Continuar" : "Entrar e sincronizar"}</Button>
    <div class="flex justify-between gap-3 text-xs"><button type="button" class="text-brand-400 hover:text-brand-300" onclick={() => openPortalAccount("register")}>Criar conta no site</button><button type="button" class="text-fg-muted hover:text-fg" onclick={() => openPortalAccount("recover")}>Esqueci a senha</button></div>
</form>
