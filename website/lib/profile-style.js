const profileAccents = ["brand", "ocean", "mint", "sunset", "rose", "lilac"];
const profileFrames = ["none", "halo", "double", "orbit"];
const profileEffects = ["none", "shimmer", "pulse"];
const profileLayouts = ["classic", "centered"];
const defaultProfileStyle = { accent: "brand", frame: "none", effect: "none", layout: "classic" };
const profileBenefits = {
  free: { favourites: 8, accents: ["brand"], frames: ["none"], effects: ["none"], layouts: ["classic"] },
  plus: { favourites: 16, accents: [...profileAccents], frames: ["none", "halo", "double"], effects: ["none"], layouts: ["classic"] },
  pro: { favourites: 24, accents: [...profileAccents], frames: [...profileFrames], effects: [...profileEffects], layouts: [...profileLayouts] }
};
function parseProfileStyle(value) {
  if (value === void 0) return { ...defaultProfileStyle };
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const record = value;
  if (Object.keys(record).some((key) => !["accent", "frame", "effect", "layout"].includes(key))) return null;
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
    effect: benefits.effects.includes(style.effect) ? style.effect : "none",
    layout: benefits.layouts.includes(style.layout) ? style.layout : "classic"
  };
}
function profileStyleAllowed(value, plan) {
  const active = activeProfileStyle(value, plan);
  return active.accent === value.accent && active.frame === value.frame && active.effect === value.effect && active.layout === value.layout;
}
function retainedProfileStyle(value, stored, plan) {
  const saved = parseProfileStyle(stored) || { ...defaultProfileStyle };
  const active = activeProfileStyle(saved, plan);
  const allowed = profileBenefits[plan];
  return {
    accent: !allowed.accents.includes(saved.accent) && value.accent === active.accent ? saved.accent : value.accent,
    frame: !allowed.frames.includes(saved.frame) && value.frame === active.frame ? saved.frame : value.frame,
    effect: !allowed.effects.includes(saved.effect) && value.effect === active.effect ? saved.effect : value.effect,
    layout: !allowed.layouts.includes(saved.layout) && value.layout === active.layout ? saved.layout : value.layout
  };
}
function isProfileGif(value) {
  return typeof value === "string" && value.startsWith("data:image/gif;base64,");
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
  profileBenefits,
  profileEffects,
  profileFrames,
  profileLayouts,
  profileStyleAllowed,
  retainedProfileImage,
  retainedProfileStyle
};
