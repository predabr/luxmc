<script lang="ts">
	import { translateUi as uiText } from "$lib/i18n/useTranslation.svelte";
	
	import { LockKeyhole, User, Eye, EyeOff, Loader2 } from "lucide-svelte";
	import { luxAccountLogin } from "$lib/api/luxAccount";
	import { openPortalAccount } from "$lib/api/system";
	import { account, saveCurrentAccount } from "$lib/stores/account.svelte";
	import type { AuthAccount } from "$lib/api/auth";
	import { activeSkinStore } from "$lib/stores/skin.svelte";

	let { onAuthenticated, busy = $bindable(false) }: { onAuthenticated?: (value: AuthAccount) => void; busy?: boolean } = $props();

	let username = $state("");
	let password = $state("");
	let reveal = $state(false);
	let error = $state("");

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (busy) return;
		error = "";
		if (!/^[A-Za-z0-9_]{3,16}$/.test(username.trim())) {
			error = uiText("ui.9d89cf7c9510c215");
			return;
		}
		if (password.length < 8) {
			error = uiText("ui.9d89cf7c9510c215");
			return;
		}
		busy = true;
		try {
			const result = await luxAccountLogin(username.trim(), password);
			password = "";
			account.value = {
				id: result.id,
				uuid: result.uuid,
				username: result.username,
				minecraftToken: "",
				expiresAt: 0,
				skinUrl: result.skinUrl ?? null,
				skinVariant: result.skinVariant ?? "classic",
				capeUrl: result.capeUrl ?? null
			};
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
			onAuthenticated?.(result);
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
</script>

<form onsubmit={submit} class="login-form">
	<label class="login-label" for="lux-user">{uiText("ui.8b1359b7132fdc1d")}</label>
	<div class="login-row">
		<div class="login-field">
			<User class="login-field-icon h-4 w-4" />
			<input
				id="lux-user"
				bind:value={username}
				required
				minlength="3"
				maxlength="16"
				autocomplete="username"
				placeholder={uiText("ui.59beae7dfca0966b")}
			/>
		</div>
	</div>

	<label class="login-label" for="lux-pass">{uiText("ui.24dd3e9c8bda27e8")}</label>
	<div class="login-field">
		<LockKeyhole class="login-field-icon h-4 w-4" />
		<input
			id="lux-pass"
			bind:value={password}
			type={reveal ? "text" : "password"}
			required
			minlength="8"
			maxlength="128"
			autocomplete="current-password"
			placeholder={uiText("ui.d0403526e4afeb8f")}
		/>
		<button type="button" class="login-reveal" aria-label={reveal ? "Ocultar senha" : "Mostrar senha"} onclick={() => (reveal = !reveal)}>
			{#if reveal}<EyeOff class="h-4 w-4" />{:else}<Eye class="h-4 w-4" />{/if}
		</button>
	</div>

	{#if error}<p role="alert" class="login-error">{error}</p>{/if}

	<button type="submit" class="login-submit" disabled={busy}>
		{#if busy}<Loader2 class="h-4 w-4 animate-spin" />{/if}
		{busy ? uiText("ui.a89d514549530e6e") : uiText("ui.a9e1a56d159026d7")}
	</button>

	<div class="login-links">
		<button type="button" onclick={() => openPortalAccount("register")}>{uiText("ui.09d1da3dfaa19d16")}</button>
		<button type="button" onclick={() => openPortalAccount("recover")}>{uiText("ui.6328c0bf79cc2aa8")}</button>
	</div>
</form>

<style>
	.login-form { display: flex; flex-direction: column; gap: 6px; width: 100%; }
	.login-label {
		margin-top: 6px;
		font-size: 10px; font-weight: 700; letter-spacing: .09em; text-transform: uppercase;
		color: rgb(var(--fg-subtle)); text-align: left;
	}
	.login-row { display: flex; align-items: stretch; gap: 10px; }
	.login-field {
		display: flex; align-items: center; gap: 9px;
		flex: 1; min-width: 0;
		padding: 0 12px;
		min-height: 42px;
		border: 1px solid rgb(var(--fg) / .22);
		border-radius: 8px;
		background: rgb(var(--bg) / .55);
		color: rgb(var(--fg-muted));
		transition: border-color 140ms ease, background-color 140ms ease;
	}
	.login-field:focus-within { border-color: rgb(var(--brand-400) / .8); background: rgb(var(--bg) / .75); }
	.login-field input {
		flex: 1; min-width: 0;
		border: none; background: transparent; outline: none;
		color: rgb(var(--fg)); font-size: 13px;
		padding-block: 10px;
	}
	.login-field input::placeholder { color: rgb(var(--fg-subtle)); }
	.login-field :global(.login-field-icon) { flex: none; }
	.login-reveal { display: grid; place-items: center; flex: none; color: rgb(var(--fg-muted)); padding: 4px; border-radius: 6px; }
	.login-reveal:hover { color: rgb(var(--fg)); }
	.login-submit {
		display: flex; align-items: center; justify-content: center; gap: 8px;
		margin-top: 12px;
		min-height: 44px;
		border: 1px solid rgb(var(--brand-500));
		border-radius: 8px;
		background: rgb(var(--brand-500));
		color: rgb(var(--brand-foreground));
		font-size: 12.5px; font-weight: 800; letter-spacing: .1em; text-transform: uppercase;
		transition: background-color 150ms ease, border-color 150ms ease;
	}
	.login-submit:hover:not(:disabled) { background: rgb(var(--brand-400)); border-color: rgb(var(--brand-400)); }
	.login-submit:disabled { opacity: .6; cursor: not-allowed; }
	.login-submit:focus-visible { outline: 2px solid rgb(var(--brand-300)); outline-offset: 2px; }
	.login-error {
		margin-top: 8px; padding: 8px 10px;
		border: 1px solid rgb(var(--danger) / .35); border-radius: 8px;
		background: rgb(var(--danger) / .1);
		font-size: 12px; line-height: 1.5; color: rgb(var(--danger));
	}
	.login-links { display: flex; justify-content: space-between; gap: 12px; margin-top: 10px; }
	.login-links button { font-size: 11.5px; font-weight: 600; color: rgb(var(--fg-muted)); }
	.login-links button:hover { color: rgb(var(--brand-300)); text-decoration: underline; text-underline-offset: 3px; }
</style>
