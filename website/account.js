const $ = (selector) => document.querySelector(selector);
let mode = "login";
let current = null;
let poll;
let generation = 0;

async function api(action, body = {}, area = "account") {
  let response;
  try {
    response = await fetch(`/api/${area}/${action}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      credentials: "same-origin",
      signal: AbortSignal.timeout(15000),
    });
  } catch (err) {
    throw new Error(
      "Falha na conexão com o servidor. Verifique se sua internet está ativa.",
    );
  }
  let result;
  try {
    result = await response.json();
  } catch {
    throw Object.assign(
      new Error(
        "O serviço de conta não respondeu como esperado. Tente novamente em instantes.",
      ),
      { status: response.status },
    );
  }
  if (!response.ok) {
    if (result.account) {
      current = result.account;
      fillPreferences();
    }
    const msg =
      response.status === 503
        ? "O serviço de conta está temporariamente indisponível. Tente novamente em instantes."
        : result.error ||
          `Erro no servidor (${response.status}). Tente novamente em instantes.`;
    throw Object.assign(new Error(msg), { status: response.status, suggestions: result.suggestions || [] });
  }
  return result;
}
function setMode(next) {
  mode = ["login", "register", "recover"].includes(next) ? next : "login";
  $("#authTitle").textContent = {
    login: "Entrar na sua conta.",
    register: "Criar sua conta.",
    recover: "Recuperar acesso.",
  }[mode];
  $("#authDescription").textContent = {
    login: "Seu nickname e sua senha. Simples assim.",
    register: "Escolha seu nickname Minecraft e uma senha segura.",
    recover: "Use o código recebido no cadastro para definir uma nova senha.",
  }[mode];
  $("#authSubmit").textContent = {
    login: "Entrar ↗",
    register: "Criar conta ↗",
    recover: "Redefinir senha ↗",
  }[mode];
  $("#recoveryField").hidden = mode !== "recover";
  $("#recoveryCode").required = mode === "recover";
  $("#recoveryCode").disabled = mode !== "recover";
  $("#confirmField").hidden = mode === "login";
  $("#confirmPassword").required = mode !== "login";
  $("#confirmPassword").disabled = mode === "login";
  $("#password").autocomplete =
    mode === "login" ? "current-password" : "new-password";
  $("#passwordLabel").textContent = mode === "recover" ? "Nova senha" : "Senha";
  $("#password").value = "";
  $("#confirmPassword").value = "";
  $("#authError").hidden = true;
  $("#password").type = "password";
  $("#revealPassword").textContent = "Mostrar";
  $("#revealPassword").setAttribute("aria-pressed", "false");
  $("#revealPassword").setAttribute("aria-label", "Mostrar senha");
  for (const tab of document.querySelectorAll(".portal-tab-btn"))
    tab.setAttribute("aria-pressed", String(tab.dataset.mode === mode));
  for (const input of document.querySelectorAll("#authForm input"))
    input.removeAttribute("aria-invalid");
}
function message(text, error = false) {
  $("#dashboardMessage").textContent = text;
  $("#dashboardMessage").classList.toggle("error", error);
}
function fillPreferences() {
  const prefs = current.preferences;
  $("#theme").value = prefs.theme || "default-dark";
  $("#accentTheme").value = prefs.accentTheme || "blue";
  $("#language").value = prefs.language || "pt-BR";
  $("#animations").checked = prefs.animations !== false;
}
function showAccount(account, recoveryCode) {
  current = account;
  $("#authPanel").hidden = true;
  $("#dashboard").hidden = false;
  $("#accountNickname").textContent = account.username;
  $("#accountAvatar").src =
    `https://mc-heads.net/avatar/${encodeURIComponent(account.username)}/128`;
  fillPreferences();
  if (recoveryCode) {
    $("#generatedRecovery").value = recoveryCode;
    $("#recoveryNotice").hidden = false;
  }
  clearTimeout(poll);
  const run = ++generation;
  void refreshFriends(run);
}
for (const button of document.querySelectorAll("[data-mode]"))
  button.addEventListener("click", () => setMode(button.dataset.mode));
let avatarTimer;
let nicknameCheck = 0;
function showNicknameSuggestions(names) {
  for (const name of names || []) {
    const suggestion = document.createElement("button");
    suggestion.type = "button";
    suggestion.className = "btn btn-secondary";
    suggestion.textContent = name;
    suggestion.addEventListener("click", () => {
      $("#nickname").value = name;
      $("#nickname").dispatchEvent(new Event("input"));
      $("#authError").hidden = true;
      $("#nickname").focus();
    });
    $("#authError").append(document.createTextNode(" "), suggestion);
  }
}
$("#nickAvatar").addEventListener(
  "error",
  (event) => {
    event.target.src = "assets/avatar-steve.png";
  },
  { once: true },
);
$("#nickname").addEventListener("input", () => {
  clearTimeout(avatarTimer);
  const check = ++nicknameCheck;
  avatarTimer = setTimeout(async () => {
    const name = $("#nickname").value.trim();
    $("#nickAvatar").src = /^[A-Za-z0-9_]{3,16}$/.test(name)
      ? `https://mc-heads.net/avatar/${name}/64`
      : "assets/avatar-steve.png";
    if (mode !== "register" || !/^[A-Za-z0-9_]{3,16}$/.test(name)) return;
    try {
      const result = await api("nickname", { username: name });
      if (check !== nicknameCheck || mode !== "register" || $("#nickname").value.trim() !== name) return;
      $("#authError").hidden = result.available;
      if (!result.available) {
        $("#authError").textContent = result.reason === "official" ? "Esse nome já pertence a um jogador oficial do Minecraft. Escolha uma sugestão:" : "Esse nome já está cadastrado. Escolha uma sugestão:";
        showNicknameSuggestions(result.suggestions);
      }
    } catch (error) {
      if (check === nicknameCheck && mode === "register") {
        $("#authError").textContent = error.message;
        $("#authError").hidden = false;
      }
    }
  }, 850);
});
$("#revealPassword").addEventListener("click", () => {
  const visible = $("#password").type === "password";
  $("#password").type = visible ? "text" : "password";
  $("#revealPassword").textContent = visible ? "Ocultar" : "Mostrar";
  $("#revealPassword").setAttribute("aria-pressed", String(visible));
  $("#revealPassword").setAttribute(
    "aria-label",
    visible ? "Ocultar senha" : "Mostrar senha",
  );
});
$("#authForm").addEventListener("submit", async (event) => {
  event.preventDefault();
  const form = $("#authForm");
  const invalid = [...form.querySelectorAll("input")].find(
    (input) => !input.validity.valid,
  );
  form
    .querySelectorAll("input")
    .forEach((input) => input.removeAttribute("aria-invalid"));
  if (invalid) {
    invalid.setAttribute("aria-invalid", "true");
    $("#authError").textContent =
      invalid.id === "nickname"
        ? "Use de 3 a 16 letras, números ou sublinhados."
        : invalid.id === "recoveryCode"
          ? "Informe seu código de recuperação."
          : "A senha precisa ter pelo menos 8 caracteres.";
    $("#authError").hidden = false;
    invalid.focus();
    return;
  }
  if (
    mode !== "login" &&
    $("#password").value !== $("#confirmPassword").value
  ) {
    $("#authError").textContent = "As senhas precisam ser iguais.";
    $("#authError").hidden = false;
    $("#confirmPassword").setAttribute("aria-invalid", "true");
    $("#confirmPassword").focus();
    return;
  }
  const button = $("#authSubmit");
  button.disabled = true;
  button.textContent = "Conectando…";
  form.setAttribute("aria-busy", "true");
  $("#authError").hidden = true;
  try {
    const result = await api(mode, {
      username: $("#nickname").value.trim(),
      password: $("#password").value,
      recoveryCode: $("#recoveryCode").value.trim(),
    });
    $("#authForm").reset();
    showAccount(result.account, result.recoveryCode);
  } catch (error) {
    $("#authError").textContent = error.message;
    $("#authError").hidden = false;
    showNicknameSuggestions(error.suggestions);
  } finally {
    form.setAttribute("aria-busy", "false");
    button.disabled = false;
    button.textContent =
      mode === "login"
        ? "Entrar ↗"
        : mode === "register"
          ? "Criar conta ↗"
          : "Redefinir senha ↗";
  }
});
$("#copyRecovery").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText($("#generatedRecovery").value);
    $("#copyRecovery").textContent = "Copiado";
  } catch {
    $("#generatedRecovery").select();
  }
});
$("#savedRecovery").addEventListener("click", () => {
  $("#recoveryNotice").hidden = true;
  $("#generatedRecovery").value = "";
});
for (const button of document.querySelectorAll("[data-panel]"))
  button.addEventListener("click", () => {
    document.querySelectorAll("[data-panel]").forEach((tab) => {
      tab.classList.toggle("active", tab === button);
      tab.setAttribute("aria-pressed", String(tab === button));
    });
    document
      .querySelectorAll("[data-dashboard-panel]")
      .forEach(
        (panel) =>
          (panel.hidden =
            panel.dataset.dashboardPanel !== button.dataset.panel),
      );
    message("");
  });
$("#preferencesForm").addEventListener("submit", async (event) => {
  event.preventDefault();
  const button = event.submitter;
  button.disabled = true;
  try {
    const result = await api("sync", {
      revision: current.revision,
      preferences: {
        theme: $("#theme").value,
        accentTheme: $("#accentTheme").value,
        language: $("#language").value,
        animations: $("#animations").checked,
      },
    });
    current = result.account;
    message(
      "Preferências salvas. Seu launcher receberá a atualização automaticamente.",
    );
  } catch (error) {
    message(error.message, true);
  } finally {
    button.disabled = false;
  }
});
$("#passwordForm").addEventListener("submit", async (event) => {
  event.preventDefault();
  const button = event.submitter;
  button.disabled = true;
  try {
    await api("password", {
      currentPassword: $("#currentPassword").value,
      password: $("#newPassword").value,
    });
    $("#passwordForm").reset();
    message("Senha atualizada. As outras sessões foram encerradas.");
  } catch (error) {
    message(error.message, true);
  } finally {
    button.disabled = false;
  }
});
$("#logoutButton").addEventListener("click", async () => {
  try {
    await api("logout");
    generation++;
    clearTimeout(poll);
    current = null;
    $("#dashboard").hidden = true;
    $("#authPanel").hidden = false;
    $("#generatedRecovery").value = "";
    $("#recoveryNotice").hidden = true;
    setMode("login");
  } catch (error) {
    message(error.message, true);
  }
});
function friendRow(friend, search = false) {
  const row = document.createElement("div");
  row.className = "friend-row";
  const avatar = document.createElement("img");
  avatar.src =
    friend.avatarUrl ||
    `https://mc-heads.net/avatar/${encodeURIComponent(friend.username)}/64`;
  avatar.alt = "";
  const label = document.createElement("div");
  label.textContent = friend.username;
  const status = document.createElement("small");
  status.textContent = search
    ? `#${friend.id.slice(0, 8)}`
    : friend.status === "pending"
      ? friend.incoming
        ? "Convite recebido"
        : "Convite enviado"
      : friend.status === "in_game"
        ? `Jogando ${friend.activity || "Minecraft"}`
        : friend.status === "online"
          ? "Online"
          : "Offline";
  label.append(status);
  row.append(avatar, label);
  const action = (name, text) => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = text;
    button.addEventListener("click", async () => {
      button.disabled = true;
      try {
        await api(name, { targetId: friend.id }, "social");
        message(name === "invite" ? "Convite enviado." : "Lista atualizada.");
        clearTimeout(poll);
        await refreshFriends(generation);
      } catch (error) {
        message(error.message, true);
      } finally {
        button.disabled = false;
      }
    });
    row.append(button);
  };
  if (search) action("invite", "Adicionar");
  else {
    if (friend.incoming && friend.status === "pending")
      action("accept", "Aceitar");
    action(
      "remove",
      friend.status === "pending" ? "Recusar / cancelar" : "Remover",
    );
  }
  return row;
}
async function refreshFriends(run) {
  if (!current || run !== generation) return;
  try {
    const result = await api("sync", { readOnly: true }, "social");
    if (run !== generation) return;
    if (result.me.avatarUrl) $("#accountAvatar").src = result.me.avatarUrl;
    $("#friendsList").replaceChildren(
      ...result.friends.map((friend) => friendRow(friend)),
    );
    if (!result.friends.length) {
      const empty = document.createElement("p");
      empty.className = "empty-friends";
      empty.textContent =
        "Sua próxima dupla está por aí. Busque um nickname para enviar um convite.";
      $("#friendsList").append(empty);
    }
  } catch (error) {
    if (run === generation) message(error.message, true);
  }
  if (run === generation && !document.hidden)
    poll = setTimeout(() => void refreshFriends(run), 25000);
}
$("#friendSearch").addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const result = await api(
      "search",
      { query: $("#friendNickname").value.trim() },
      "social",
    );
    $("#searchResults").replaceChildren(
      ...result.users.map((friend) => friendRow(friend, true)),
    );
    if (!result.users.length) message("Nenhum jogador encontrado.");
  } catch (error) {
    message(error.message, true);
  }
});
setMode(new URLSearchParams(location.search).get("mode") || "login");
api("me")
  .then((result) => showAccount(result.account))
  .catch(() => {});
document.addEventListener("visibilitychange", () => {
  clearTimeout(poll);
  if (!document.hidden && current) void refreshFriends(generation);
});
window.addEventListener("pagehide", () => {
  generation++;
  clearTimeout(poll);
  clearTimeout(avatarTimer);
});
