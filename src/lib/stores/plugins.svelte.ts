import { pluginsApi, type PluginPermission, type PluginRecord, type PluginRegistry, type RegistryPlugin } from "$lib/api/plugins";

const COMMUNITY_REGISTRIES = ["https://raw.githubusercontent.com/predabr/luxmc-community-plugins/main/registry.json"];

type PluginToast = (message: string, kind?: "success" | "error" | "info") => void;

export const covers = $state<Record<string, string>>({});

class PluginsStore {
	records = $state<PluginRecord[]>([]);
	registry = $state<PluginRegistry | null>(null);
	loading = $state(false);
	registryLoading = $state(false);
	busyId = $state<string | null>(null);
	scanning = $state(false);
	error = $state<string | null>(null);
	catalogUrl = $state<string | null>(null);
	notify: PluginToast = () => {};

	enabled = $derived(this.records.filter((record) => record.enabled && record.scan.clean));
	blocked = $derived(this.records.filter((record) => !record.scan.clean));

	async refresh() {
		this.loading = true;
		try {
			this.records = await pluginsApi.list();
			this.error = null;
			await this.loadCovers();
		} catch (error) {
			this.error = String(error);
		} finally {
			this.loading = false;
		}
	}

	async loadRegistry(url?: string | null) {
		this.registryLoading = true;
		try {
			const base = await pluginsApi.registry(url ?? this.catalogUrl);
			const merged = [...base.plugins];
			for (const source of COMMUNITY_REGISTRIES) {
				try {
					const extra = await pluginsApi.registry(source);
					for (const plugin of extra.plugins) {
						if (!merged.some((entry) => entry.id === plugin.id)) merged.push({ ...plugin, tags: [...(plugin.tags || []), 'comunidade'] });
					}
				} catch { /* catalogo da comunidade opcional */ }
			}
			this.registry = { ...base, plugins: merged };
			this.error = null;
		} catch (error) {
			this.error = String(error);
		} finally {
			this.registryLoading = false;
		}
	}

	async init() {
		await Promise.all([this.refresh(), this.loadRegistry()]);
	}

	installedId(id: string) {
		return this.records.some((record) => record.manifest.id === id);
	}

	private track(id: string, run: () => Promise<void>) {
		this.busyId = id;
		return run().finally(() => { this.busyId = null; });
	}

	async install(plugin: RegistryPlugin) {
		if (!plugin.downloadUrl) {
			this.notify("Este plugin não informa um link de download.", "error");
			return;
		}
		await this.track(plugin.id, async () => {
			try {
				await pluginsApi.install(plugin.id, plugin.downloadUrl!, plugin.sha256 ?? null);
				await this.refresh();
				this.notify(`${plugin.name} instalado e aprovado pelo Shield.`, "success");
			} catch (error) {
				this.notify(`Shield recusou ${plugin.name}: ${String(error)}`, "error");
			}
		});
	}

	async installUrl(url: string) {
		await this.track(url, async () => {
			try {
				const record = await pluginsApi.install(url, url, null);
				await this.refresh();
				this.notify(`${record.manifest.name} instalado e aprovado pelo Shield.`, "success");
			} catch (error) {
				this.notify(`Shield recusou o plugin: ${String(error)}`, "error");
			}
		});
	}

	async remove(id: string) {
		await this.track(id, async () => {
			try {
				await pluginsApi.uninstall(id);
				await this.refresh();
				this.notify("Plugin removido.", "success");
			} catch (error) {
				this.notify(String(error), "error");
			}
		});
	}

	async toggle(id: string, enabled: boolean) {
		await this.track(id, async () => {
			try {
				await pluginsApi.setEnabled(id, enabled);
				await this.refresh();
			} catch (error) {
				await this.refresh();
				this.notify(String(error), "error");
			}
		});
	}

	async create(payload: { id: string; name: string; author: string; description: string; permissions: PluginPermission[]; source: string }) {
		this.busyId = payload.id;
		try {
			const record = await pluginsApi.create(payload);
			await this.refresh();
			this.notify(`${record.manifest.name} criado e aprovado pelo Shield.`, "success");
			return record;
		} catch (error) {
			this.notify(String(error), "error");
			return null;
		} finally {
			this.busyId = null;
		}
	}

	async saveSource(id: string, source: string) {
		this.busyId = id;
		try {
			const record = await pluginsApi.updateSource(id, source);
			await this.refresh();
			this.notify("Codigo salvo e aprovado pelo Shield.", "success");
			return record;
		} catch (error) {
			await this.refresh();
			this.notify(String(error), "error");
			return null;
		} finally {
			this.busyId = null;
		}
	}

	async setCover(id: string, path: string, premium: boolean) {
		try {
			await pluginsApi.setCover(id, path, premium);
			await this.loadCovers();
			await this.refresh();
			this.notify("Capa atualizada.", "success");
		} catch (error) {
			this.notify(String(error), "error");
		}
	}

	async clearCover(id: string) {
		try {
			await pluginsApi.clearCover(id);
			delete covers[id];
			await this.refresh();
		} catch (error) {
			this.notify(String(error), "error");
		}
	}

	async publish(id: string) {
		try {
			const published = await pluginsApi.publish(id);
			await navigator.clipboard.writeText(published.entryJson).catch(() => {});
			this.notify("Pacote gerado e entrada do catalogo copiada.", "success");
			return published;
		} catch (error) {
			this.notify(String(error), "error");
			return null;
		}
	}

	async loadCovers() {
		await Promise.all(this.records.map(async (record) => {
			if (!record.cover) { delete covers[record.manifest.id]; return; }
			try {
				const url = await pluginsApi.cover(record.manifest.id);
				if (url) covers[record.manifest.id] = url; else delete covers[record.manifest.id];
			} catch { delete covers[record.manifest.id]; }
		}));
	}

	async scanAll() {
		this.scanning = true;
		try {
			this.records = await pluginsApi.rescan(null);
			this.notify("Varredura concluída pelo Luxmc Shield.", "success");
		} catch (error) {
			this.notify(String(error), "error");
		} finally {
			this.scanning = false;
		}
	}

	async rescan(id: string) {
		await this.track(id, async () => {
			try {
				this.records = await pluginsApi.rescan(id);
			} catch (error) {
				this.notify(String(error), "error");
			}
		});
	}

	async openFolder() {
		try {
			await pluginsApi.openFolder();
		} catch (error) {
			this.notify(String(error), "error");
		}
	}
}

export const plugins = new PluginsStore();
