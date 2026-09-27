import { api } from "./client";
export interface TunnelStatus { mode: "host" | "client"; invitation: string | null; localAddress: string | null; expiresAt: number; pingMs: number | null; transport: string }
export const hostWorld = (port: number): Promise<TunnelStatus> => api.invoke("host_world", { port });
export const joinTunnel = (invitation: string): Promise<TunnelStatus> => api.invoke("join_world", { invitation });
export const stopSession = (): Promise<void> => api.invoke("stop_session");
export const tunnelStatus = (): Promise<TunnelStatus | null> => api.invoke("tunnel_status");
