import { api } from './client';
export const ownerTools = (accountId: string, profileId: string, operation: 'diagnose' | 'repair'): Promise<Record<string, unknown>> => api.invoke('owner_tools', { accountId, profileId, operation });
