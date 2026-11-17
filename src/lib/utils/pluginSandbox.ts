import type { PluginManifest, PluginRecord } from "$lib/api/plugins";

export type SandboxManifest = Pick<PluginManifest, "id" | "name" | "version" | "author" | "permissions">;

export const PLUGIN_TEMPLATE_JS = `// Plugin Luxmc — JavaScript puro, roda isolado (sandbox)
// API disponivel: window.Luxmc
//
//   Luxmc.ui.mount(html)   -> desenha o HTML do teu painel
//   Luxmc.ui.append(html)  -> adiciona mais HTML no fim
//   Luxmc.ui.toast(msg)    -> mostra um aviso do launcher
//   Luxmc.storage.get(k)   -> Promise<string | null>
//   Luxmc.storage.set(k,v) -> salva um dado do plugin (64 KB)
//   Luxmc.plugin           -> { id, name, version, author, permissions }

async function render() {
	const aberto = await Luxmc.storage.get("aberto");
	Luxmc.ui.mount(\`
		<div style="display:flex;flex-direction:column;gap:12px">
			<h2 style="margin:0">Ola do \${Luxmc.plugin.name}!</h2>
			<p style="margin:0;color:#a1a1aa">Este painel e HTML + CSS + JS rodando dentro do Luxmc.</p>
			<button id="botao" style="align-self:flex-start;height:34px;padding:0 14px;border-radius:10px;border:0;background:#3b82f6;color:#fff;font-weight:600;cursor:pointer">
				Clicar
			</button>
			<small style="color:#71717a">Ultimo clique: \${aberto ?? "nunca"}</small>
		</div>
	\`);
	document.getElementById("botao")?.addEventListener("click", () => {
		const agora = new Date().toLocaleTimeString();
		Luxmc.storage.set("aberto", agora);
		Luxmc.ui.toast("Funcionou! " + agora);
	});
}

render();
`;

export const PLUGIN_TEMPLATE_HTML = `<div class="painel">
	<h2>Ola do plugin</h2>
	<p>Escreva HTML aqui e use Luxmc.ui.mount() para trocar o conteudo.</p>
</div>

<style>
	.painel { display: grid; gap: 8px; }
	.painel h2 { margin: 0; }
</style>
`;

export function buildSandboxDocument(manifest: SandboxManifest, script: string) {
	const payload = JSON.stringify({
		id: manifest.id,
		name: manifest.name,
		version: manifest.version || "1.0.0",
		author: manifest.author,
		permissions: manifest.permissions,
	});
	const safeScript = script.replace(new RegExp("<" + "/script", "gi"), "<" + "\/script");
	const openStyle = "<" + "style>";
	const closeStyle = "<" + "/style>";
	const openScript = "<" + "script>";
	const closeScript = "<" + "/script>";
	const template = [
		'<!doctype html><html lang="pt-BR"><head><meta charset="utf-8" />',
		'<meta name="viewport" content="width=device-width, initial-scale=1" />',
		openStyle,
		":root{color-scheme:dark}*{box-sizing:border-box}",
		'body{margin:0;padding:16px;font-family:system-ui,-apple-system,"Segoe UI",sans-serif;background:transparent;color:#e7e7ea;font-size:13px;line-height:1.5}',
		"h1,h2,h3{margin:0 0 8px;font-size:15px}button{font:inherit}",
		"code,pre{font-family:ui-monospace,Consolas,monospace;font-size:12px}",
		closeStyle + '</head><body><div id="luxmc-plugin-root"></div>',
		openScript,
		"(() => {",
		"const meta = " + payload + ";",
		'const send = (type, data) => parent.postMessage({ source: "luxmc-plugin", type, payload: data }, "*");',
		'const root = document.getElementById("luxmc-plugin-root");',
		"const pending = new Map();",
		"window.Luxmc = {",
		"  plugin: meta,",
		'  ui: { mount(html){ root.innerHTML = String(html ?? ""); }, append(html){ root.insertAdjacentHTML("beforeend", String(html ?? "")); }, toast(message){ send("toast", String(message)); }, resize(){ send("resize", { height: document.documentElement.scrollHeight }); } },',
		'  storage: { get(key){ return new Promise((resolve) => { pending.set(String(key), resolve); send("storage:get", String(key)); }); }, set(key, value){ send("storage:set", { key: String(key), value: String(value) }); return Promise.resolve(true); } },',
		'  log(...args){ send("log", args.map(String).join(" ")); }',
		"};",
		'window.addEventListener("message", (event) => {',
		"  const data = event.data;",
		'  if (!data || data.source !== "luxmc-host") return;',
		'  if (data.type === "storage:value") { const resolve = pending.get(data.payload.key); if (resolve) { pending.delete(data.payload.key); resolve(data.payload.value); } }',
		"});",
		"try {",
		"__SCRIPT__",
		'} catch (error) { send("error", String(error)); root.innerHTML = "<p style=\\"color:#f87171\\">Falha no plugin: " + String(error) + "</p>"; }',
		"})();",
		closeScript + "</body></html>",
	].join("\n");
	return template.replace("__SCRIPT__", () => safeScript);
}

export function buildSandboxFromRecord(record: PluginRecord, script: string) {
	return buildSandboxDocument(record.manifest, script);
}

export function slugifyPluginId(value: string) {
	return value
		.normalize("NFD")
		.replace(/[\u0300-\u036f]/g, "")
		.toLowerCase()
		.replace(/[^a-z0-9._-]+/g, "-")
		.replace(/^[-.]+|[-.]+$/g, "")
		.slice(0, 60);
}
