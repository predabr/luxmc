function texture(value, cape = false) {
  if (value === null && cape)
    return null;
  if (typeof value !== "string" || value.length > 18e4 || !value.startsWith("data:image/png;base64,"))
    throw Object.assign(new Error("Envie uma textura PNG v\xE1lida."), { status: 400 });
  let bytes;
  try {
    bytes = Uint8Array.from(atob(value.slice(22)), (c) => c.charCodeAt(0));
  } catch {
    throw Object.assign(new Error("PNG inv\xE1lido."), { status: 400 });
  }
  if (bytes.length < 33 || bytes.length > 131072 || ![137, 80, 78, 71, 13, 10, 26, 10].every((n, i) => bytes[i] === n))
    throw Object.assign(new Error("PNG inv\xE1lido."), { status: 400 });
  const view = new DataView(bytes.buffer);
  const width = view.getUint32(16), height = view.getUint32(20);
  if (cape ? width < 2 || width > 512 || height * 2 !== width : width !== 64 || ![32, 64].includes(height))
    throw Object.assign(new Error("Dimens\xF5es da textura inv\xE1lidas."), { status: 400 });
  return value;
}
export {
  texture
};
