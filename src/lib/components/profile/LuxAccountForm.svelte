<script lang="ts">
import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
    import { button as launcherButton } from "$lib/components/ui/button";
    import { ArrowLeft, ArrowRight, LockKeyhole, Cloud } from "lucide-svelte";
    import { luxAccountLogin } from "$lib/api/luxAccount";
    import { openPortalAccount } from "$lib/api/system";
    import { account, saveCurrentAccount } from "$lib/stores/account.svelte";
    import type { AuthAccount } from "$lib/api/auth";
    import { activeSkinStore } from "$lib/stores/skin.svelte";
    import Button from "$lib/components/ui/Button.svelte";
    let { onAuthenticated, busy = $bindable(false) }: { onAuthenticated?: (value: AuthAccount) => void; busy?: boolean } = $props();
    let username = $state("");
    let password = $state("");
    let step = $state<"nickname" | "password">("nickname");
    let error = $state("");
    async function submit(event: SubmitEvent) {
        event.preventDefault();
        if (busy) return;
        error = "";
        if (step === "nickname") {
            if (!/^[A-Za-z0-9_]{3,16}$/.test(username.trim())) { error = uiText("ui.9d89cf7c9510c215"); return; }
            username = username.trim(); step = "password"; return;
        }
        busy = true;
        try {
            const result = await luxAccountLogin(username, password);
            password = "";
            account.value = { id: result.id, uuid: result.uuid, username: result.username, minecraftToken: "", expiresAt: 0, skinUrl: result.skinUrl ?? null, skinVariant: result.skinVariant ?? "classic", capeUrl: result.capeUrl ?? null };
            void saveCurrentAccount(account.value);
            activeSkinStore.setSkin({
                id: result.uuid,
                name: result.username,
                url: `https://mc-heads.net/body/${result.username}/300`,
                skinUrl: result.skinUrl || `https://minotar.net/skin/${result.username}`,
                avatarUrl: `https://mc-heads.net/avatar/${result.username}/100`,
                type: result.skinVariant?.toLowerCase() === "slim" ? "alex" : "steve",
                hasCape: Boolean(result.capeUrl),
                capeType: result.capeUrl ? "custom" : "none",
                customCapeUrl: result.capeUrl || ""
            });
            busy = false;
            onAuthenticated?.(result);
        } catch (cause) { error = String(cause); }
        finally { busy = false; }
    }
</script>
<form onsubmit={submit} class="space-y-4 py-4">
    <div class="flex items-center gap-3 rounded-2xl border border-brand-500/20 bg-brand-500/5 p-4">
        <Cloud class="h-6 w-6 text-brand-400" />
        <div><h2 class="text-sm font-bold text-fg">{uiText("ui.805a081e4bf8c49b")}</h2><p class="mt-1 text-xs text-fg-muted">{uiText("ui.61cb39640b462b9b")}</p></div>
    </div>
    {#if step === "nickname"}
        <label for="lux-nickname" class="block text-xs font-semibold text-fg-muted">{uiText("ui.8b1359b7132fdc1d")}</label>
        <input id="lux-nickname" bind:value={username} required minlength="3" maxlength="16" autocomplete="username" placeholder={uiText("ui.59beae7dfca0966b")} class="w-full rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm text-fg focus:border-brand-400 focus:outline-none" />
    {:else}
        <div class="flex items-center gap-3"><img loading="lazy" decoding="async" src={`https://mc-heads.net/avatar/${username}/64`} alt="" class="h-10 w-10 rounded-xl" /><span class="text-sm font-bold">{username}</span><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "ml-auto" })} disabled={busy} onclick={() => { step = "nickname"; password = ""; error = ""; }}><ArrowLeft class="inline h-3 w-3" /> {uiText("ui.a5575a1a9fa7978f")}</button></div>
        <input type="text" name="username" value={username} autocomplete="username" class="sr-only" tabindex="-1" aria-label={uiText("ui.d720f61c8c5e281e")} readonly />
        <label for="lux-password" class="block text-xs font-semibold text-fg-muted">{uiText("ui.24dd3e9c8bda27e8")}</label>
        <input id="lux-password" bind:value={password} type="password" required minlength="8" maxlength="128" autocomplete="current-password" placeholder={uiText("ui.d0403526e4afeb8f")} class="w-full rounded-xl border border-border bg-bg-subtle px-4 py-3 text-sm text-fg focus:border-brand-400 focus:outline-none" />
    {/if}
    {#if error}<p role="alert" class="rounded-xl border border-danger/20 bg-danger/10 p-3 text-xs text-danger">{error}</p>{/if}
    <Button type="submit" variant="primary" block loading={busy}>{#if !busy}{#if step === "password"}<LockKeyhole class="h-4 w-4" />{:else}<ArrowRight class="h-4 w-4" />{/if}{/if}{step === "nickname" ? uiText("common.next") : uiText("ui.a9e1a56d159026d7")}</Button>
    <div class="flex justify-between gap-3 text-xs"><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "" })} onclick={() => openPortalAccount("register")}>{uiText("ui.09d1da3dfaa19d16")}</button><button type="button" class={launcherButton({ variant: "ghost", size: "sm", class: "" })} onclick={() => openPortalAccount("recover")}>{uiText("ui.6328c0bf79cc2aa8")}</button></div>
</form>
