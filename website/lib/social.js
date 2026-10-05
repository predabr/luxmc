export async function friendSnapshot(db, id, now = Math.floor(Date.now() / 1000)) {
  const { results } = await db.prepare("SELECT u.id, u.username, u.avatar_url, u.last_seen, u.activity, u.instance_name, u.mc_version, u.loader, u.server_host, u.server_port, r.accepted, r.recipient FROM social_relationships r JOIN social_users u ON u.id = CASE WHEN r.sender = ? THEN r.recipient ELSE r.sender END WHERE (r.sender = ? OR r.recipient = ?) AND NOT EXISTS (SELECT 1 FROM social_blocks b WHERE (b.owner = ? AND b.target = u.id) OR (b.target = ? AND b.owner = u.id)) LIMIT 200")
    .bind(id, id, id, id, id).all();
  return results.map(row => {
    const online = row.accepted === 1 && row.last_seen > now - 75;
    return { id: row.id, username: row.username, avatarUrl: row.avatar_url || null,
      status: row.accepted ? (online ? row.activity : "offline") : "pending",
      incoming: !row.accepted && row.recipient === id,
      lastSeen: row.accepted && row.last_seen ? new Date(row.last_seen * 1000).toISOString() : null,
      activity: online ? row.instance_name : null,
      mcVersion: online ? row.mc_version : null, loader: online ? row.loader : null,
      serverIp: online ? row.server_host : null, serverPort: online ? row.server_port : null };
  });
}
