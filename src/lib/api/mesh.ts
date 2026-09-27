import { api } from "./client";
export interface MeshPeer { id: string; name: string; ip: string; online: boolean; transport: string }
export interface MeshStatus { available: boolean; state: string; ip: string | null; peers: MeshPeer[] }
export function meshStatus(): Promise<MeshStatus> { return api.invoke("mesh_status"); }
export function meshPing(ip: string): Promise<number> { return api.invoke("mesh_ping", { ip }); }
