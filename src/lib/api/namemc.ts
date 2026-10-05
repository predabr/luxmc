import { api, listen } from "./client";

export interface NameMcSkin {
    skinId: string;
    skinUrl: string;
}

export function nameMcOpenPicker(): Promise<void> {
    return api.invoke("namemc_open_picker");
}

export function nameMcClosePicker(): Promise<void> {
    return api.invoke("namemc_close_picker");
}

export function nameMcImportSkin(url: string): Promise<NameMcSkin> {
    return api.invoke("namemc_import_skin", { url });
}

export function listenNameMcSelection(handler: (url: string) => void) {
    return listen<string>("namemc-skin-selected", event => handler(event.payload));
}
