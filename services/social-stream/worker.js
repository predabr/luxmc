import { DurableObject } from "cloudflare:workers";
import { friendSnapshot } from "../../website/lib/social.js";

export class PresenceStream extends DurableObject {
  async fetch(request) {
    const user = request.headers.get("X-Luxmc-User");
    if (!user || request.headers.get("Upgrade")?.toLowerCase() !== "websocket") return new Response("Forbidden", { status: 403 });
    await this.ctx.storage.put("user", user);
    const sockets = this.ctx.getWebSockets();
    if (sockets.length >= 4) sockets[0].close(1008, "Connection limit");
    const pair = new WebSocketPair();
    this.ctx.acceptWebSocket(pair[1]);
    await this.sendSnapshot(user);
    await this.ctx.storage.setAlarm(Date.now() + 5000);
    return new Response(null, { status: 101, webSocket: pair[0] });
  }
  async sendSnapshot(user) {
    const message = JSON.stringify({ friends: await friendSnapshot(this.env.SOCIAL_DB, user) });
    const previous = await this.ctx.storage.get("snapshot");
    for (const socket of this.ctx.getWebSockets()) {
      const sent = socket.deserializeAttachment();
      if (message !== previous || !sent) {
        try { socket.send(message); socket.serializeAttachment(true); }
        catch { socket.close(1011, "Connection ended"); }
      }
    }
    await this.ctx.storage.put("snapshot", message);
  }
  async alarm() {
    if (!this.ctx.getWebSockets().length) { await this.ctx.storage.deleteAll(); return; }
    const user = await this.ctx.storage.get("user");
    try { if (user) await this.sendSnapshot(user); }
    finally { if (this.ctx.getWebSockets().length) await this.ctx.storage.setAlarm(Date.now() + 5000); }
  }
  webSocketMessage(socket) { socket.close(1008, "Read-only stream"); }
  webSocketError(socket) { socket.close(1011, "Stream error"); }
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname !== "/connect" || request.headers.get("Upgrade")?.toLowerCase() !== "websocket") return new Response("Not found", { status: 404 });
    const ticket = url.searchParams.get("ticket");
    if (!ticket || ticket.length !== 72) return new Response("Unauthorized", { status: 401 });
    const hash = Array.from(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(ticket))), b => b.toString(16).padStart(2, "0")).join("");
    const user = await env.SOCIAL_DB.prepare("DELETE FROM social_stream_tickets WHERE token_hash = ? AND expires_at > ? RETURNING user_id")
      .bind(hash, Math.floor(Date.now() / 1000)).first();
    if (!user) return new Response("Unauthorized", { status: 401 });
    const id = env.PRESENCE.idFromName(user.user_id);
    const internal = new Request(request);
    internal.headers.set("X-Luxmc-User", user.user_id);
    return env.PRESENCE.get(id).fetch(internal);
  }
};
