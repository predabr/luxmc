function field(id, value) {
  const length = new TextEncoder().encode(value).length;
  if (!length || length > 99) throw new Error("Campo Pix inv\xE1lido.");
  return `${id}${String(length).padStart(2, "0")}${value}`;
}
function crc16(value) {
  let crc = 65535;
  for (const byte of new TextEncoder().encode(value)) {
    crc ^= byte << 8;
    for (let bit = 0; bit < 8; bit++) crc = (crc << 1 ^ (crc & 32768 ? 4129 : 0)) & 65535;
  }
  return crc.toString(16).toUpperCase().padStart(4, "0");
}
function validPixKey(key) {
  return typeof key === "string" && key.length <= 77 && (/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(key) || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(key) || /^(?:\d{11}|\d{14}|\+[1-9]\d{7,14})$/.test(key));
}
function orderTxid(id) {
  if (!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(id)) throw new Error("Pedido Pix inv\xE1lido.");
  return BigInt(`0x${id.replaceAll("-", "")}`).toString(36).padStart(25, "0");
}
function pixPayload({ key, amount, txid = "***", name = "Luxmc", city = "SAOPAULO" }) {
  if (!validPixKey(key) || !/^(?:[a-zA-Z0-9]{1,25}|\*{3})$/.test(txid) || !/^[\x20-\x7e]{1,25}$/.test(name) || !/^[\x20-\x7e]{1,15}$/.test(city)) throw new Error("Dados Pix inv\xE1lidos.");
  if (amount !== void 0 && (!Number.isSafeInteger(amount) || amount <= 0 || amount > 99999999999)) throw new Error("Valor Pix inv\xE1lido.");
  const data = field("00", "01") + field("26", field("00", "br.gov.bcb.pix") + field("01", key)) + field("52", "0000") + field("53", "986") + (amount === void 0 ? "" : field("54", (amount / 100).toFixed(2))) + field("58", "BR") + field("59", name) + field("60", city) + field("62", field("05", txid)) + "6304";
  return data + crc16(data);
}
export {
  crc16,
  orderTxid,
  pixPayload,
  validPixKey
};
