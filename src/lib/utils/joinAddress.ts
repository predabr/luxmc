export function parseJoinAddress(value: string): { host: string; port: number } {
	const raw = value.trim().replace(/^luxmc:\/\/join\//, "");
	if (!raw || /[\s/@?#\\]/.test(raw) || /^LUX-/i.test(raw)) throw new Error("Use um endereço IP:porta ou link luxmc://join/IP:porta.");
	const match = raw.match(/^(\[[a-fA-F0-9:]+\]|[a-zA-Z0-9](?:[a-zA-Z0-9.-]*[a-zA-Z0-9])?)(?::(\d{1,5}))?$/);
	if (!match) throw new Error("Endereço de conexão inválido.");
	const port = Number(match[2] || 25565);
	if (port < 1 || port > 65535) throw new Error("Porta inválida.");
	const host = match[1];
    if (host.startsWith("[")) {
        try { new URL(`http://${host}:${port}`); } catch { throw new Error("Endereço IPv6 inválido."); }
    } else if (!host.split(".").every(label => /^[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$/.test(label)) || host.length > 253) {
        throw new Error("Nome de servidor inválido.");
    }
    return { host, port };
}

