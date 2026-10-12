function field(id: string, value: string) {
    const length = new TextEncoder().encode(value).length;
    if (!length || length > 99) throw new Error('Campo Pix inválido.');
    return `${id}${String(length).padStart(2, '0')}${value}`;
}
export function crc16(value: string) {
    let crc = 65535;
    for (const byte of new TextEncoder().encode(value)) {
        crc ^= byte << 8;
        for (let bit = 0; bit < 8; bit++) crc = ((crc << 1) ^ ((crc & 32768) ? 4129 : 0)) & 65535;
    }
    return crc.toString(16).toUpperCase().padStart(4, '0');
}
export function validPixKey(key: unknown): key is string {
    return typeof key === 'string' && key.length <= 77 && (/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(key) || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(key) || /^(?:\d{11}|\d{14}|\+[1-9]\d{7,14})$/.test(key));
}
export function orderTxid(id: string) {
    if (!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/i.test(id)) throw new Error('Pedido Pix inválido.');
    return BigInt(`0x${id.replaceAll('-', '')}`).toString(36).padStart(25, '0');
}
export function pixPayload({ key, amount, txid = '***', name = 'Luxmc', city = 'SAOPAULO' }: { key: string; amount?: number; txid?: string; name?: string; city?: string }) {
    if (!validPixKey(key) || !/^(?:[a-zA-Z0-9]{1,25}|\*{3})$/.test(txid) || !/^[\x20-\x7e]{1,25}$/.test(name) || !/^[\x20-\x7e]{1,15}$/.test(city)) throw new Error('Dados Pix inválidos.');
    if (amount !== undefined && (!Number.isSafeInteger(amount) || amount <= 0 || amount > 99999999999)) throw new Error('Valor Pix inválido.');
    const data = field('00', '01') + field('26', field('00', 'br.gov.bcb.pix') + field('01', key)) + field('52', '0000') + field('53', '986') + (amount === undefined ? '' : field('54', (amount / 100).toFixed(2))) + field('58', 'BR') + field('59', name) + field('60', city) + field('62', field('05', txid)) + '6304';
    return data + crc16(data);
}
