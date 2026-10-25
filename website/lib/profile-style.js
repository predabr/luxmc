const profileAccents = ["brand", "ocean", "mint", "sunset", "rose", "lilac"];
const profileFrames = ["none", "halo", "double", "orbit"];
const profileEffects = ["none", "shimmer", "pulse"];
const profileAmbientEffects = ["none", "aurora", "beacon"];
const profileLayouts = ["classic", "centered"];
const profileFrameDetails = ["none", "prism"];
const profileShowcaseLayouts = ["grid", "spotlight", "poster"];
const profileCardBackgrounds = ["none", "cover"];
const defaultProfileStyle = { accent: "brand", frame: "none", effect: "none", layout: "classic" };
const profileBenefits = {
  free: { favourites: 8, accents: ["brand"], frames: ["none"], effects: ["none"], ambientEffects: ["none"], layouts: ["classic"], frameDetails: ["none"], showcaseLayouts: ["grid"], cardBackgrounds: ["none"] },
  plus: { favourites: 16, accents: [...profileAccents], frames: ["none", "halo", "double"], effects: ["none"], ambientEffects: ["none"], layouts: ["classic"], frameDetails: ["none"], showcaseLayouts: ["grid"], cardBackgrounds: ["none"] },
  pro: { favourites: 24, accents: [...profileAccents], frames: [...profileFrames], effects: [...profileEffects], ambientEffects: [...profileAmbientEffects], layouts: [...profileLayouts], frameDetails: [...profileFrameDetails], showcaseLayouts: [...profileShowcaseLayouts], cardBackgrounds: [...profileCardBackgrounds] }
};
function parseProfileStyle(value) {
  if (value === void 0) return { ...defaultProfileStyle };
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const record = value;
  if (Object.keys(record).some((key) => !["accent", "frame", "effect", "layout", "frameDetail", "cardBackground", "showcaseLayout", "ambientEffect"].includes(key))) return null;
  if (record.ambientEffect !== void 0 && !profileAmbientEffects.includes(record.ambientEffect)) return null;
  if (record.showcaseLayout !== void 0 && !profileShowcaseLayouts.includes(record.showcaseLayout)) return null;
  if (record.cardBackground !== void 0 && !profileCardBackgrounds.includes(record.cardBackground)) return null;
  if (record.frameDetail !== void 0 && !profileFrameDetails.includes(record.frameDetail)) return null;
  const style = { ...defaultProfileStyle, ...record };
  if (!profileAccents.includes(style.accent) || !profileFrames.includes(style.frame) || !profileEffects.includes(style.effect) || !profileLayouts.includes(style.layout)) return null;
  return style;
}
function activeProfileStyle(value, plan) {
  const style = parseProfileStyle(value) || { ...defaultProfileStyle };
  const benefits = profileBenefits[plan];
  return {
    accent: benefits.accents.includes(style.accent) ? style.accent : "brand",
    frame: benefits.frames.includes(style.frame) ? style.frame : "none",
    ...style.ambientEffect !== void 0 ? { ambientEffect: benefits.ambientEffects.includes(style.ambientEffect) ? style.ambientEffect : "none" } : {},
    effect: benefits.effects.includes(style.effect) ? style.effect : "none",
    layout: benefits.layouts.includes(style.layout) ? style.layout : "classic",
    ...style.showcaseLayout !== void 0 ? { showcaseLayout: benefits.showcaseLayouts.includes(style.showcaseLayout) ? style.showcaseLayout : "grid" } : {},
    ...style.frameDetail !== void 0 ? { frameDetail: benefits.frameDetails.includes(style.frameDetail) ? style.frameDetail : "none" } : {},
    ...style.cardBackground !== void 0 ? { cardBackground: benefits.cardBackgrounds.includes(style.cardBackground) ? style.cardBackground : "none" } : {}
  };
}
function profileStyleAllowed(value, plan) {
  const active = activeProfileStyle(value, plan);
  return active.ambientEffect === value.ambientEffect && active.accent === value.accent && active.frame === value.frame && active.effect === value.effect && active.layout === value.layout && active.cardBackground === value.cardBackground && active.frameDetail === value.frameDetail && active.showcaseLayout === value.showcaseLayout;
}
function retainedProfileStyle(value, stored, plan) {
  const saved = parseProfileStyle(stored) || { ...defaultProfileStyle };
  const active = activeProfileStyle(saved, plan);
  const allowed = profileBenefits[plan];
  return {
    accent: !allowed.accents.includes(saved.accent) && value.accent === active.accent ? saved.accent : value.accent,
    frame: !allowed.frames.includes(saved.frame) && value.frame === active.frame ? saved.frame : value.frame,
    ...value.ambientEffect !== void 0 || saved.ambientEffect !== void 0 ? { ambientEffect: value.ambientEffect === void 0 || saved.ambientEffect !== "none" && plan !== "pro" && value.ambientEffect === "none" ? saved.ambientEffect : value.ambientEffect } : {},
    effect: !allowed.effects.includes(saved.effect) && value.effect === active.effect ? saved.effect : value.effect,
    layout: !allowed.layouts.includes(saved.layout) && value.layout === active.layout ? saved.layout : value.layout,
    ...value.showcaseLayout !== void 0 || saved.showcaseLayout !== void 0 ? { showcaseLayout: value.showcaseLayout === void 0 || saved.showcaseLayout !== "grid" && plan !== "pro" && value.showcaseLayout === "grid" ? saved.showcaseLayout : value.showcaseLayout } : {},
    ...value.frameDetail !== void 0 || saved.frameDetail !== void 0 ? { frameDetail: value.frameDetail === void 0 || saved.frameDetail === "prism" && plan !== "pro" && value.frameDetail === "none" ? saved.frameDetail : value.frameDetail } : {},
    ...value.cardBackground !== void 0 || saved.cardBackground !== void 0 ? { cardBackground: value.cardBackground === void 0 || saved.cardBackground === "cover" && plan !== "pro" && value.cardBackground === "none" ? saved.cardBackground : value.cardBackground } : {}
  };
}
function isProfileGif(value) {
  if (typeof value !== "string") return false;
  if (value.startsWith("data:image/gif;base64,")) return true;
  const prefix = "data:image/webp;base64,";
  if (!value.startsWith(prefix)) return false;
  try {
    const header = atob(value.slice(prefix.length, prefix.length + 40));
    return header.startsWith("RIFF") && header.slice(8, 12) === "WEBP" && header.slice(12, 16) === "VP8X" && (header.charCodeAt(20) & 2) !== 0;
  } catch {
    return false;
  }
}
function activeProfileImage(value, plan) {
  return plan === "free" && isProfileGif(value) ? "" : value;
}
function retainedProfileImage(value, stored, plan) {
  return plan === "free" && !value && isProfileGif(stored) ? stored : value;
}
export {
  activeProfileImage,
  activeProfileStyle,
  defaultProfileStyle,
  isProfileGif,
  parseProfileStyle,
  profileAccents,
  profileAmbientEffects,
  profileBenefits,
  profileCardBackgrounds,
  profileEffects,
  profileFrameDetails,
  profileFrames,
  profileLayouts,
  profileShowcaseLayouts,
  profileStyleAllowed,
  retainedProfileImage,
  retainedProfileStyle
};
