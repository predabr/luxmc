function collections(value) {
  if (value === void 0) return [];
  if (!Array.isArray(value) || value.length > 6) return null;
  const ids = /* @__PURE__ */ new Set();
  for (const item of value) {
    if (!item || typeof item !== "object" || typeof item.id !== "string" || !/^[a-zA-Z0-9-]{1,64}$/.test(item.id) || ids.has(item.id) || typeof item.title !== "string" || !item.title.trim() || item.title.length > 80 || typeof item.description !== "string" || item.description.length > 240 || !Array.isArray(item.entries) || item.entries.length > 8) return null;
    ids.add(item.id);
    for (const entry of item.entries) {
      if (!entry || !["modrinth", "curseforge"].includes(entry.source) || typeof entry.name !== "string" || !entry.name.trim() || entry.name.length > 200 || ![entry.projectId, entry.versionId].every((id) => typeof id === "string" && /^[a-zA-Z0-9-]{1,80}$/.test(id))) return null;
      if (entry.source === "curseforge" && ![entry.projectId, entry.versionId].every((id) => /^[1-9][0-9]*$/.test(id))) return null;
    }
  }
  return value;
}
function profileImage(value) {
  if (value === "") return true;
  if (typeof value !== "string" || value.length > 934e3) return false;
  const match = /^data:image\/(png|jpeg|webp|gif);base64,([A-Za-z0-9+/]+={0,2})$/.exec(value);
  if (!match) return false;
  try {
    const bytes = atob(match[2]);
    if (bytes.length > 7e5 || bytes.length < 12) return false;
    return match[1] === "png" ? bytes.startsWith("\x89PNG\r\n\n") : match[1] === "jpeg" ? bytes.startsWith("\xFF\xD8\xFF") : match[1] === "gif" ? /^GIF8[79]a/.test(bytes) : bytes.startsWith("RIFF") && bytes.slice(8, 12) === "WEBP";
  } catch {
    return false;
  }
}
function validatePublicProfile(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const body = value;
  const shared = collections(body.collections);
  if (!shared) return null;
  if (body.displayName !== void 0 && (typeof body.displayName !== "string" || body.displayName.length > 32)) return null;
  if (body.status !== void 0 && (typeof body.status !== "string" || body.status.length > 80)) return null;
  if (typeof body.description !== "string" || body.description.length > 400 || !profileImage(body.banner) || !profileImage(body.portrait)) return null;
  if (!Array.isArray(body.packs) || body.packs.length > 8 || body.packs.some((pack) => typeof pack !== "string" || !pack.trim() || pack.length > 80)) return null;
  const packs = body.packs;
  return { collections: shared, displayName: typeof body.displayName === "string" ? body.displayName.trim() : "", status: typeof body.status === "string" ? body.status.trim() : "", description: body.description.trim(), banner: body.banner, portrait: body.portrait, packs: [...new Set(packs.map((pack) => pack.trim()))] };
}
export {
  collections,
  profileImage,
  validatePublicProfile
};
