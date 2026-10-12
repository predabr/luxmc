import { activeProfileStyle } from './profile-style.js';
import type { Database } from "./types.js";
export async function friendSnapshot(db: Database, id: any, now: any = Math.floor(Date.now() / 1000), ownerId?: string) {
    const { results } = await db.prepare("SELECT u.id, u.username, u.avatar_url, u.last_seen, u.activity, u.instance_name, u.mc_version, u.loader, u.server_host, u.server_port, r.accepted, r.recipient, m.plan AS membership_plan, m.expires_at AS membership_expires, COALESCE(t.tags, '[]') AS tags, p.profile_style FROM social_relationships r JOIN social_users u ON u.id = CASE WHEN r.sender = ? THEN r.recipient ELSE r.sender END LEFT JOIN social_public_profiles p ON p.user_id = u.id LEFT JOIN social_user_tags t ON t.user_id = u.id LEFT JOIN lux_memberships m ON m.social_id = u.id WHERE (r.sender = ? OR r.recipient = ?) AND u.removed_at = 0 AND NOT EXISTS (SELECT 1 FROM social_blocks b WHERE (b.owner = ? AND b.target = u.id) OR (b.target = ? AND b.owner = u.id)) LIMIT 200")
        .bind(id, id, id, id, id).all();
    return results.map((row: any) => {
        const online = row.accepted === 1 && row.last_seen > now - 75;
        const plan = row.membership_expires > now ? row.membership_plan : 'free';
        return { profileStyle: activeProfileStyle(row.accepted ? JSON.parse(row.profile_style || '{}') : undefined, plan), membership: {plan: row.membership_expires > now ? row.membership_plan : 'free', expiresAt: row.membership_expires || null}, id: row.id, username: row.username, avatarUrl: row.avatar_url || null, role: ownerId && row.id === ownerId ? 'owner' : 'member', tags: JSON.parse(row.tags),
            status: row.accepted ? (online ? row.activity : "offline") : "pending",
            incoming: !row.accepted && row.recipient === id,
            lastSeen: row.accepted && row.last_seen ? new Date(row.last_seen * 1000).toISOString() : null,
            activity: online ? row.instance_name : null,
            mcVersion: online ? row.mc_version : null, loader: online ? row.loader : null,
            serverIp: online ? row.server_host : null, serverPort: online ? row.server_port : null };
    });
}
