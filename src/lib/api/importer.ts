import { open } from '@tauri-apps/plugin-dialog';
import { api } from './client';

export interface ExternalInstance {
    launcher: string;
    name: string;
    path: string;
    mcVersion: string;
    loader: string;
    modCount: number;
    hasSaves: boolean;
}

export async function chooseImportDirectory(): Promise<ExternalInstance | null> {
    const path = await open({ directory: true, multiple: false });
    if (typeof path !== 'string') return null;
    return api.invoke('importer_inspect_directory', { path });
}
