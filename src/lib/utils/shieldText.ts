import type { ShieldThreat } from '$lib/api/types';
import { translateUi } from '$lib/i18n/useTranslation.svelte';

const codes = ['known_malware', 'fractureiser', 'suspicious_structure', 'discord_webhook', 'scan_incomplete'];
export function shieldThreatText(threat: ShieldThreat, description = false): string {
    if (!threat.code || !codes.includes(threat.code)) return description ? threat.description : threat.threatType;
    const label = translateUi(`shield.${threat.code}${description ? 'Help' : ''}`);
    return description && threat.code === 'scan_incomplete' ? `${label} ${threat.description}` : label;
}
