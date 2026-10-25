import QRCode from 'qrcode';
import type { ProfileStyle } from '../site-server/lib/profile-style';

type Plan = 'free' | 'plus' | 'pro';
interface Preset { id: string; name: string; preferences: Record<string, string | number | boolean> }
interface Order { id: string; socialId: string; username: string; displayName: string; plan: 'plus' | 'pro'; amount: number; bank: string; pixKey: string; pixCode: string | null; pixReference: string; days: number; status: string; version: number; createdAt: number; updatedAt: number; note: string; receiptHash: string | null; receiptMime: string | null; receiptUploadedAt: number | null }
interface Snapshot { profile?: {profileStyle:ProfileStyle;displayName:string;status:string}; identity: { id: string; username: string; owner: boolean }; csrfToken?: string | null; security?: { twoFactor: boolean; verifiedUntil: number; expiresAt: number } | null; membership: { plan: Plan; expiresAt: number | null; collections: number; presets: number }; checkout: { enabled: boolean; bank: string; pixKey: string }; settings: { preferences: Record<string, string | number | boolean>; presets: Preset[]; revision: number }; orders: Order[] }
interface GrantUser { id: string; username: string; display_name: string; friendCode: string; plan: Plan; expires_at: number | null; revision: number }
function localText(key: string, fallback: string): string { const value = window.LuxI18n?.t(key); return typeof value === 'string' && value !== key ? value : fallback; }
const ownerPage = document.body.dataset.commerce === 'owner';
const element = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const money = (amount: number) => (amount / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
const date = (seconds: number) => new Date(seconds * 1000).toLocaleString('pt-BR');
const planName = (plan: Plan) => `Luxmc ${plan === 'free' ? 'Free' : plan === 'plus' ? 'Plus' : 'Pro'}`;
function enteredAmount(value: string) { const parts = /^(\d{1,9})(?:[.,](\d{1,2}))?$/.exec(value.trim()); return parts ? Number(parts[1]) * 100 + Number((parts[2] || '').padEnd(2, '0')) : null; }
function updateApprovalButtons() { for (const form of document.querySelectorAll<HTMLFormElement>('[data-payment-review]')) { const approve = form.querySelector<HTMLButtonElement>('[data-approve]')!; approve.disabled = busy || !form.querySelector<HTMLInputElement>('[data-credit-confirmed]')!.checked || !/^[a-zA-Z0-9]{32}$/.test(form.querySelector<HTMLInputElement>('[data-transaction]')!.value.trim()) || enteredAmount(form.querySelector<HTMLInputElement>('[data-received]')!.value) !== Number(form.dataset.paymentReview); } }
const statuses: Record<string, string> = { awaiting_receipt: 'Aguardando comprovante', pending: 'Esperando verificação', approved: 'Aprovado', rejected: 'Rejeitado', cancelled: 'Cancelado' };
let snapshot: Snapshot | null = null;
let accountResolved = false;
let busy = false;
let offset = 0;
let queueTotal = 0;
let selectedReceipt = '';
let csrfToken = '';
let ownerTab = 'orders';
let lastInteraction = Date.now();
let lastHeartbeat = Date.now();
let unlocking: Promise<void> | null = null;
let refreshGeneration = 0;
let lastUpdateRevision = '';
let updatesRunning = false;
let updatesTimer: ReturnType<typeof setTimeout> | undefined;
let visibleOrders: Order[] = [];
const receiptRequests = new Map<string, AbortController>();
const receiptUrls = new Map<string, string>();
function headers() { return { 'Content-Type': 'application/json', ...(csrfToken ? { 'X-Luxmc-CSRF': csrfToken } : {}) }; }
async function request<T = any>(action: string, body: unknown = {}): Promise<T> {
    for (let attempt = 0; attempt < 2; attempt++) {
        const response = await fetch(`/api/commerce/${action}`, { method: 'POST', credentials: 'same-origin', headers: headers(), body: JSON.stringify(body), signal: AbortSignal.timeout(15000) });
        const value = await response.json();
        if (response.status === 428 && ownerPage && attempt === 0) { await unlockOwner(); continue; }
        if (!response.ok) {
            if (response.status === 401 || (response.status === 403 && action.startsWith('owner_'))) { accountResolved = true; lock(); }
            throw new Error(value.error || 'Não foi possível concluir.');
        }
        if (typeof value.csrfToken === 'string') csrfToken = value.csrfToken;
        return value;
    }
    throw new Error('Confirme novamente seu acesso.');
}
function unlockOwner(): Promise<void> {
    if (unlocking) return unlocking;
    const dialog = element<HTMLDialogElement>('owner-unlock');
    const password = element<HTMLInputElement>('unlock-password');
    const otp = element<HTMLInputElement>('unlock-otp');
    const submit = element<HTMLButtonElement>('unlock-submit');
    const cancel = element<HTMLButtonElement>('unlock-cancel');
    element('unlock-error').textContent = ''; password.value = ''; otp.value = '';
    submit.disabled = false; cancel.disabled = false;
    unlocking = new Promise<void>((resolve, reject) => {
        const cancelled = () => { dialog.close(); password.value = ''; otp.value = ''; unlocking = null; lock(); reject(new Error('Confirmação cancelada. Nenhuma alteração foi feita.')); };
        cancel.onclick = cancelled;
        dialog.oncancel = event => { event.preventDefault(); cancelled(); };
        element<HTMLFormElement>('unlock-form').onsubmit = async event => {
            event.preventDefault(); submit.disabled = true; cancel.disabled = true;
            try {
                const response = await fetch('/api/commerce/owner_unlock', { method: 'POST', credentials: 'same-origin', headers: headers(), body: JSON.stringify({ password: password.value, otp: otp.value }), signal: AbortSignal.timeout(15000) });
                password.value = ''; otp.value = '';
                const result = await response.json();
                if (!response.ok) throw new Error(result.error || 'Não foi possível confirmar.');
                csrfToken = result.csrfToken; lastInteraction = Date.now(); dialog.close(); unlocking = null; resolve();
            } catch (error) { element('unlock-error').textContent = (error as Error).message; }
            finally { submit.disabled = false; cancel.disabled = false; }
        };
    });
    dialog.showModal(); password.focus(); return unlocking;
}
function node<K extends keyof HTMLElementTagNameMap>(tag: K, text = '', className = ''): HTMLElementTagNameMap[K] { const result = document.createElement(tag); result.textContent = text; if (className) result.className = className; return result; }
function message(text = '', error = false) { const target = ownerPage && !snapshot ? element('login-message') : !ownerPage && element<HTMLDialogElement>('checkout').open ? element('checkout-message') : element('message'); target.replaceChildren(); if (text) target.append(node('p', text, `notice${error ? ' error' : ''}`)); }
function button(text: string, action: () => void | Promise<void>, style = 'secondary', onError?: (error: string) => void) { const result = node('button', text, `commerce-button ${style}`); result.type = 'button'; result.onclick = () => void perform(action, onError); return result; }
function updatePlanAvailability() {
    for (const control of document.querySelectorAll<HTMLButtonElement>('[data-plan]')) {
        const pending = snapshot?.orders.find(order => order.plan === control.dataset.plan && ['awaiting_receipt', 'pending'].includes(order.status));
        const current = snapshot?.membership.plan === control.dataset.plan;
        control.disabled = busy || !accountResolved || (!pending && (current || Boolean(snapshot && !snapshot.checkout.enabled)));
        control.textContent = !accountResolved ? localText('plans.loading','Carregando…') : pending ? localText('plans.order','Ver pedido') : current ? localText('plans.active','Seu plano ativo') : !snapshot ? localText('plans.login','Entrar para escolher') : !snapshot.checkout.enabled ? localText('plans.paused','Novos pedidos pausados') : control.dataset.plan === 'plus' ? localText('plans.choosePlus','Escolher Plus ↗') : localText('plans.choosePro','Escolher Pro ↗');
        control.title = current && !pending ? 'Você já tem este plano ativo' : pending ? 'Abrir seu pedido em andamento' : control.textContent || '';
    }
}
async function perform(action: () => unknown | Promise<unknown>, onError?: (error: string) => void) {
    if (busy) return; busy = true;
    const controls = new Map(Array.from(document.querySelectorAll<HTMLButtonElement>('button'), control => [control, control.disabled]));
    controls.forEach((_, control) => control.disabled = true);
    try { await action(); } catch (error) { const text = String((error as Error).message); if (onError) onError(text); else message(text, true); }
    finally { busy = false; controls.forEach((disabled, control) => { if (control.isConnected) control.disabled = disabled; }); updatePlanAvailability(); updateApprovalButtons(); pagination(); }
}
function lock() {
    refreshGeneration++;
    for (const id of receiptRequests.keys()) clearReceipt(id);
    for (const id of receiptUrls.keys()) clearReceipt(id);
    snapshot = null;
    visibleOrders = []; lastUpdateRevision = '';
    updatePlanAvailability();
    selectedReceipt = ''; csrfToken = '';
    for (const id of ['owner-view', 'owner-account', 'orders', 'preferences', 'profile-customization']) { const target = document.getElementById(id); if (target) target.hidden = true; }
    const identity = element('identity'); identity.hidden = false;
    identity.replaceChildren(node('p', ownerPage ? 'ACESSO EXCLUSIVO' : 'SUA CONTA', 'eyebrow'), node('h2', ownerPage ? 'Acesso reservado ao dono' : 'Conecte sua conta'));
    identity.append(node('p', ownerPage ? 'Entre com a senha da sua central para gerenciar o Luxmc.' : 'Use a conta Luxmc do launcher. Para contas Microsoft ou locais, abra “Meu plano” no launcher para continuar com a mesma identidade.'));
    if (ownerPage) {
        const form = node('form'); const password = labelledInput(form, 'Senha do painel', '', 'password'); password.minLength = 14; password.maxLength = 128; password.autocomplete = 'current-password';
        password.required = true;
        const otp = labelledInput(form, 'Código do autenticador ou de recuperação (se ativado)'); otp.maxLength = 40; otp.autocomplete = 'one-time-code';
        form.append(button('Entrar no painel', async () => { if (!form.reportValidity()) return; const secret = password.value; const code = otp.value; password.value = ''; otp.value = ''; await request('owner_login', {password:secret,otp:code}); lastInteraction = Date.now(); element('login-message').replaceChildren(); await refresh(); }, ''));
        form.onsubmit = event => { event.preventDefault(); form.querySelector<HTMLButtonElement>('button')?.click(); }; identity.append(form);
    }
    if (!ownerPage) { const link = node('a', 'Entrar na conta Luxmc', 'commerce-button'); link.href = '/conta'; identity.append(link); }
    element('order-list').replaceChildren();
    if (!ownerPage) element<HTMLDialogElement>('checkout').close();
    if (!ownerPage) {
        element('checkout-availability').hidden = true;
        for (const card of document.querySelectorAll<HTMLElement>('[data-tier]')) { card.dataset.current = 'false'; card.querySelector<HTMLElement>('.plan-current')!.hidden = true; }
    }
    if (ownerPage) { for (const id of ['audit','owner-settings','owner-security','owner-account','stats']) element(id).replaceChildren(); }
    if (ownerPage) element('manual-grant').replaceChildren();
}
function labelledInput(parent: HTMLElement, title: string, value = '', type = 'text') { const label = node('label', title); const input = node('input'); input.type = type; input.value = value; label.append(input); parent.append(label); return input; }
function clearReceipt(id: string) {
    receiptRequests.get(id)?.abort(); receiptRequests.delete(id);
    const url = receiptUrls.get(id); if (url) URL.revokeObjectURL(url); receiptUrls.delete(id);
}
function renderReceipt(order: Order, container: HTMLElement, owner: boolean) {
    const content = node('div'); container.append(content);
    const load = async () => {
        clearReceipt(order.id);
        const controller = new AbortController(); receiptRequests.set(order.id, controller);
        content.replaceChildren(node('p', 'Carregando comprovante…', 'payment-hint'));
        const options = {credentials: 'same-origin' as const, signal: AbortSignal.any([controller.signal, AbortSignal.timeout(15000)]), cache: 'no-store' as const};
        try {
            let response = await fetch(`/api/receipts/${order.id}?v=${order.version}`, options);
            if (response.status === 428 && owner) { await unlockOwner(); response = await fetch(`/api/receipts/${order.id}?v=${order.version}`, options); }
            if (!response.ok) { const result = await response.json().catch(() => ({})); throw new Error(result.error || 'Não foi possível abrir o comprovante. Tente novamente.'); }
            const blob = await response.blob();
            if (controller.signal.aborted || selectedReceipt !== order.id || !container.isConnected) return;
            if (blob.size > 1048576 || !['image/png', 'image/jpeg', 'image/webp', 'application/pdf'].includes(blob.type)) throw new Error('O comprovante não veio em um formato válido.');
            const url = URL.createObjectURL(blob); receiptUrls.set(order.id, url);
            if (blob.type === 'application/pdf') {
                const download = node('a', 'Baixar comprovante PDF', 'commerce-button secondary'); download.href = url; download.download = 'comprovante.pdf'; content.replaceChildren(download);
            } else {
                const image = node('img'); image.alt = `Comprovante do pedido ${order.id}`; image.className = 'receipt-image'; image.src = url;
                image.onerror = () => content.replaceChildren(node('p', 'Não foi possível exibir esta imagem. Você pode tentar carregar novamente.'), button('Carregar novamente', load));
                content.replaceChildren(image);
            }
        } catch (error) {
            if (controller.signal.aborted || selectedReceipt !== order.id || !container.isConnected) return;
            content.replaceChildren(node('p', (error as Error).name === 'TimeoutError' ? 'O comprovante demorou para responder. Tente novamente.' : (error as Error).message, 'notice error'), button('Carregar novamente', load));
        } finally { if (receiptRequests.get(order.id) === controller) receiptRequests.delete(order.id); }
    };
    void load();
}
function syncOrders(orders: Order[], owner: boolean) {
    visibleOrders = orders;
    if (selectedReceipt && !orders.some(order => order.id === selectedReceipt)) { clearReceipt(selectedReceipt); selectedReceipt = ''; }
    const list = element('order-list');
    const existing = new Map(Array.from(list.querySelectorAll<HTMLElement>(':scope > [data-order]'), card => [card.dataset.order!, card]));
    const next = orders.map(order => {
        const signature = JSON.stringify([order, selectedReceipt === order.id]);
        const previous = existing.get(order.id);
        if (previous?.dataset.signature === signature) return previous;
        clearReceipt(order.id);
        const card = renderOrder(order, owner); card.dataset.signature = signature;
        return card;
    });
    const retained = new Set(next);
    for (const child of Array.from(list.children)) if (!retained.has(child as HTMLElement)) { const id = (child as HTMLElement).dataset.order; if (id && !orders.some(order => order.id === id)) clearReceipt(id); child.remove(); }
    let cursor = list.firstChild;
    for (const card of next) { if (card === cursor) cursor = cursor.nextSibling; else list.insertBefore(card, cursor); }
}
function renderOrder(order: Order, owner: boolean) {
    const card = node('article', '', 'order-card'); card.dataset.order = order.id;
    const top = node('div', '', 'order-top'); const heading = node('div'); heading.append(node('h3', `${planName(order.plan)} · ${money(order.amount)}`));
    if (owner) heading.append(node('p', order.displayName ? `${order.displayName} (@${order.username})` : order.username), node('p', `Conta: ${order.username}#${order.socialId.slice(0, 8)} · ${order.socialId}`, 'order-code'));
    heading.append(node('p', `Pedido ${order.id}`, 'order-code')); top.append(heading, node('span', statuses[order.status] || order.status, `status-chip ${order.status}`)); card.append(top, node('p', `Criado em ${date(order.createdAt)} · ${order.days} dias a partir da aprovação`));
    if (order.note) card.append(node('p', order.note));
    if (!owner && ['awaiting_receipt', 'pending'].includes(order.status) && snapshot) {
        const pay = node('div', '', 'checkout-block');
        const steps = node('ol', '', 'payment-steps');
        for (const [index, title] of ['Pagar com Pix', 'Enviar comprovante', 'Conferência e liberação'].entries()) steps.append(node('li', title, index === (order.status === 'pending' ? 2 : 0) ? 'current' : ''));
        pay.append(steps, node('p', `Este plano será liberado para ${order.username}#${order.socialId.slice(0, 8)}.`, 'checkout-account'));
        if (order.status === 'pending') pay.append(node('p', 'Comprovante recebido. Aguarde a conferência do crédito no banco; não pague novamente.', 'payment-pending'));
        const details = node('details', '', 'pix-details'); details.open = order.status !== 'pending'; details.append(node('summary', order.status === 'pending' ? 'Consultar QR Code e dados do pagamento' : `Pix · ${order.bank}`));
        const grid = node('div', '', 'pix-payment-grid'); const qr = node('div', '', 'pix-qr-frame'); const instructions = node('div', '', 'pix-instructions');
        if (order.pixCode) {
            const canvas = node('canvas'); canvas.setAttribute('role', 'img'); canvas.setAttribute('aria-label', `QR Code Pix de ${money(order.amount)} para ${planName(order.plan)}`); qr.append(canvas);
            void QRCode.toCanvas(canvas, order.pixCode, { width: 256, margin: 4, errorCorrectionLevel: 'M' }).catch(() => { qr.replaceChildren(node('p', 'Use o Pix Copia e Cola ao lado.')); });
            instructions.append(node('p', 'VALOR DO PLANO', 'eyebrow'), node('strong', money(order.amount), 'pix-total'), node('p', 'Abra o app do seu banco e escaneie o QR Code. No celular, use Pix Copia e Cola.'));
            instructions.append(button('Copiar código Pix', async () => { await navigator.clipboard.writeText(order.pixCode!); message('Código Pix copiado. Confira o destinatário e o valor no app do banco antes de confirmar.'); }, ''));
            const code = node('textarea'); code.readOnly = true; code.value = order.pixCode; code.rows = 3; code.className = 'pix-copy-code'; code.setAttribute('aria-label', 'Pix Copia e Cola'); instructions.append(code);
        } else instructions.append(node('p', 'QR Code indisponível para este pedido. Use a chave Pix e confira o valor no banco.'));
        grid.append(qr, instructions); details.append(grid, node('p', 'Confira o destinatário e o valor no seu banco antes de pagar. Escanear ou copiar o código não faz a transferência.', 'payment-hint'));
        const fallback = node('details', '', 'pix-key-details'); fallback.append(node('summary', 'Pagar pela chave Pix'), node('p', order.pixKey, 'pix-key'), button('Copiar chave Pix', async () => { await navigator.clipboard.writeText(order.pixKey); message('Chave copiada. Informe o valor exato do plano no banco.'); })); details.append(fallback); pay.append(details);
        const upload = node('div', '', 'payment-upload'); upload.append(node('strong', order.receiptHash ? 'Atualizar comprovante' : 'Já pagou? Envie o comprovante'), node('p', 'Conferência manual: o plano só será ativado depois que o dono confirmar o crédito no banco. Enviar uma imagem não confirma o pagamento.'), node('p', 'Envie o comprovante emitido pelo banco: PNG, JPEG, WebP ou PDF, até 1 MB.'));
        const input = labelledInput(upload, order.receiptHash ? 'Substituir comprovante (PNG, JPEG, WebP ou PDF · até 1 MB)' : 'Enviar comprovante (PNG, JPEG, WebP ou PDF · até 1 MB)', '', 'file'); input.accept = 'image/png,image/jpeg,image/webp,application/pdf';
        upload.append(button('Enviar para verificação', async () => {
            const file = input.files?.[0]; if (!file) throw new Error('Escolha o comprovante.'); if (file.size > 1048576) throw new Error('O comprovante precisa ter até 1 MB.');
            const response = await fetch(`/api/receipts/${order.id}`, { method: 'POST', credentials: 'same-origin', headers: { ...headers(), 'Content-Type': file.type }, body: file, signal: AbortSignal.timeout(30000) });
            const result = await response.json(); if (!response.ok) throw new Error(result.error);
            message('Comprovante enviado. Seu plano será liberado após a conferência do crédito no banco.'); await refresh();
        }, ''));
        pay.append(upload);
        pay.append(button('Cancelar pedido', async () => { if (!confirm('Cancelar este pedido? Se você já pagou, procure o atendimento antes de cancelar.')) return; await request('cancel', { id: order.id }); await refresh(); }, 'danger')); card.append(pay);
    }
    if (order.status === 'approved') { const confirmation = node('div', '', 'payment-confirmed'); confirmation.append(node('strong', 'Seu plano foi liberado'), node('p', 'O recibo Luxmc já está disponível para você e para o dono. A assinatura acompanha esta conta no launcher.')); const link = node('a', 'Ver recibo Luxmc', 'commerce-button'); link.href = `/api/purchase-receipts/${order.id}`; link.target = '_blank'; link.rel = 'noopener noreferrer'; confirmation.append(link); card.append(confirmation); }
    if (order.receiptHash) card.append(button(selectedReceipt === order.id ? 'Fechar comprovante' : 'Ver comprovante', () => { if (selectedReceipt) clearReceipt(selectedReceipt); selectedReceipt = selectedReceipt === order.id ? '' : order.id; syncOrders(visibleOrders, owner); }));
    if (selectedReceipt === order.id && order.receiptHash) {
        const preview = node('div', '', 'receipt-preview');
        renderReceipt(order, preview, owner);
        preview.append(node('p', `Enviado em ${date(order.receiptUploadedAt!)}. Visível apenas para o comprador e o dono.`));
        if (owner && order.status === 'pending') {
            const form = node('form'); form.dataset.paymentReview = String(order.amount); form.append(node('strong', 'Conferência do pagamento'), node('p', `Abra o extrato do ${order.bank}. Uma imagem enviada pelo comprador pode ser falsa e não prova que o dinheiro entrou.`, 'payment-hint'), node('p', `Esperado: ${money(order.amount)} · ${planName(order.plan)} · ${order.username}#${order.socialId.slice(0, 8)}`, 'review-summary'), node('p', `Referência do QR Code: ${order.pixReference}`, 'order-code'));
            const received = labelledInput(form, 'Valor recebido no banco (R$)'); received.inputMode = 'decimal'; received.placeholder = (order.amount / 100).toFixed(2).replace('.', ','); received.dataset.received = ''; received.maxLength = 12;
            const transaction = labelledInput(form, 'Identificador da transação Pix (EndToEndId)'); transaction.maxLength = 32; transaction.autocomplete = 'off'; transaction.dataset.transaction = ''; transaction.placeholder = 'Cole os 32 caracteres do Pix no extrato';
            const note = labelledInput(form, 'Observação para o comprador / motivo da rejeição'); note.maxLength = 240; note.placeholder = 'Para rejeitar, informe o motivo que o comprador verá.';
            const feedback = node('p', '', 'notice error'); feedback.setAttribute('role', 'alert'); feedback.hidden = true;
            const label = node('label'); const checked = node('input'); checked.type = 'checkbox'; checked.dataset.creditConfirmed = ''; label.append(checked, document.createTextNode('Conferi o valor e o crédito desta transação no extrato do banco.')); form.append(label);
            const actions = node('div', '', 'toolbar');
            const review = async (decision: string) => {
                feedback.hidden = true;
                if (decision === 'reject' && !note.value.trim()) { note.focus(); throw new Error('Informe o motivo da rejeição. O comprador receberá essa mensagem.'); }
                if (decision === 'approve' && (!checked.checked || enteredAmount(received.value) !== order.amount)) throw new Error('Confira o crédito e informe o valor exato recebido no banco.');
                const accountCode = `${order.username}#${order.socialId.slice(0, 8)}`;
                if (!confirm(decision === 'approve' ? `Confirmar ${money(order.amount)} recebidos e liberar ${planName(order.plan)} por ${order.days} dias para ${accountCode}?` : `Rejeitar este pedido de ${accountCode}?`)) return;
                await request('owner_review', { id: order.id, version: order.version, receiptHash: order.receiptHash, decision, transactionId: transaction.value.trim(), receivedAmount: enteredAmount(received.value), creditConfirmed: checked.checked, note: note.value });
                clearReceipt(order.id); selectedReceipt = '';
                message(decision === 'approve' ? 'Plano liberado e disponível para sincronização no launcher.' : 'Pedido rejeitado. O motivo está disponível para o comprador.'); await refresh();
            };
            const reviewError = (text: string) => { feedback.textContent = text; feedback.hidden = false; };
            const approve = button('Aprovar e liberar plano', () => review('approve'), '', reviewError); approve.dataset.approve = ''; approve.disabled = true;
            actions.append(approve, button('Rejeitar pedido', () => review('reject'), 'danger', reviewError)); form.append(actions, feedback); form.onsubmit = event => event.preventDefault(); form.oninput = updateApprovalButtons; form.onchange = updateApprovalButtons; preview.append(form);
        }
        card.append(preview);
    }
    return card;
}
function renderPlanSummary() {
    if (!snapshot) return;
    const identity = element('identity'); identity.hidden = false;
    const details = node('div'); details.append(node('p', localText('plans.yourPlan','SEU PLANO'), 'eyebrow'), node('h2', `${snapshot.profile?.displayName || snapshot.identity.username} · ${planName(snapshot.membership.plan)}`));
    details.append(node('p', snapshot.membership.plan === 'free' ? localText('plans.always','Seu launcher completo, sempre com você.') : `${localText('plans.until','Ativo até')} ${date(snapshot.membership.expiresAt!)}`));
    const approved = snapshot.orders.find(order => order.status === 'approved');
    if (approved) { const receipt = node('a', 'Meu recibo Luxmc', 'membership-receipt'); receipt.href = `/api/purchase-receipts/${approved.id}`; receipt.target = '_blank'; receipt.rel = 'noopener noreferrer'; details.append(receipt); }
    const badge = node('span', snapshot.membership.plan === 'free' ? 'Free' : 'Assinatura ativa', 'membership-active');
    identity.replaceChildren(details, badge);
    const availability = element('checkout-availability'); availability.hidden = snapshot.checkout.enabled;
    availability.textContent = 'Novos pedidos estão pausados. Seu plano ativo continua válido.';
    for (const card of document.querySelectorAll<HTMLElement>('[data-tier]')) {
        const current = card.dataset.tier === snapshot.membership.plan;
        card.dataset.current = String(current);
        card.querySelector<HTMLElement>('.plan-current')!.hidden = !current;
    }
    syncOrders(snapshot.orders, false);
    updatePlanAvailability();
}
function openCheckout() {
    const dialog = element<HTMLDialogElement>('checkout');
    element('checkout-message').replaceChildren();
    if (!dialog.open) dialog.showModal();
}
function pagination() { if (!ownerPage) return; element<HTMLButtonElement>('previous').disabled = busy || offset === 0; element<HTMLButtonElement>('next').disabled = busy || offset + 50 >= queueTotal; element('pagination').textContent = `${queueTotal ? offset + 1 : 0}–${Math.min(offset + 50, queueTotal)} de ${queueTotal}`; }
function showOwnerTab(tab: string) {
    const titles: Record<string, [string, string]> = { orders: ['Pedidos', 'Confira os comprovantes e libere os planos em um só lugar.'], plans: ['Planos', 'Encontre a conta certa e ajuste o plano com segurança.'], history: ['Histórico', 'Veja o que mudou, quando e em qual conta.'], security: ['Segurança', 'Proteja seu acesso e controle o recebimento.'] };
    if (!titles[tab]) return;
    if (ownerTab !== tab) element('message').replaceChildren();
    ownerTab = tab; element('section-title').textContent = titles[tab][0]; element('section-description').textContent = titles[tab][1];
    for (const panel of document.querySelectorAll<HTMLElement>('[data-owner-panel]')) panel.hidden = panel.dataset.ownerPanel !== tab;
    for (const control of document.querySelectorAll<HTMLButtonElement>('[data-owner-tab]')) { if (control.dataset.ownerTab === tab) control.setAttribute('aria-current', 'page'); else control.removeAttribute('aria-current'); }
    if (tab !== 'orders' && selectedReceipt) { clearReceipt(selectedReceipt); selectedReceipt = ''; syncOrders(visibleOrders, true); }
}
function renderSecurity(passwordConfigured: boolean) {
    const section = element('owner-security');
    section.replaceChildren(node('h2', 'Proteção do acesso'), node('p', 'A senha confirma suas ações. As sessões terminam em 15 minutos e o painel se bloqueia após 5 minutos sem uso.'));
    const facts = node('div', '', 'security-facts');
    for (const [title, detail] of [['Identidade verificada','Só o ID do dono pode acessar'], ['Ações protegidas','Nova confirmação após 5 minutos'], ['Comprovantes privados','Visíveis só ao comprador e ao dono']]) { const fact = node('div'); fact.append(node('span', '✓', 'security-check'), node('strong', title), node('p', detail)); facts.append(fact); } section.append(facts);
    const mfa = node('div', '', 'security-block');
    mfa.append(node('h3', 'Autenticação em duas etapas'), node('p', snapshot?.security?.twoFactor ? 'Ativada. O acesso também exige seu autenticador ou um código de recuperação.' : 'Adicione o código do seu celular à senha. A ativação só acontece depois que você confirmar o primeiro código.'), node('span', snapshot?.security?.twoFactor ? 'Ativada' : 'Disponível para ativar', `status-chip ${snapshot?.security?.twoFactor ? 'approved' : 'pending'}`));
    if (!snapshot?.security?.twoFactor) {
        const form = node('form'); const password = labelledInput(form, 'Confirme sua senha para configurar duas etapas', '', 'password'); password.minLength = 14; password.maxLength = 128; password.required = true; password.autocomplete = 'current-password';
        const setup = node('div');
        form.append(button('Configurar duas etapas', async () => {
            if (!form.reportValidity()) return;
            const secret = password.value; password.value = ''; form.setAttribute('data-editing', 'true');
            const result = await request<{secret:string;uri:string}>('owner_totp_begin', {password:secret});
            setup.replaceChildren(node('h3', 'Conecte seu autenticador'), node('p', 'Escaneie este QR no aplicativo de autenticação do seu celular. Você também pode usar a chave manual abaixo.'));
            const qr = node('img'); qr.src = await QRCode.toDataURL(result.uri, {width:200,margin:2,errorCorrectionLevel:'M'}); qr.alt = 'QR para configurar o autenticador da central'; qr.className = 'authenticator-qr'; setup.append(qr);
            const manual = labelledInput(setup, 'Chave manual do autenticador', result.secret); manual.readOnly = true; manual.className = 'secret-code';
            const otp = labelledInput(setup, 'Primeiro código de seis números'); otp.inputMode = 'numeric'; otp.pattern = '[0-9]{6}'; otp.maxLength = 6; otp.autocomplete = 'one-time-code';
            const feedback = node('div'); feedback.setAttribute('role', 'status');
            setup.append(button('Ativar duas etapas', async () => {
                if (!/^\d{6}$/.test(otp.value)) throw new Error('Digite os seis números do autenticador.');
                const enabled = await request<{recoveryCodes:string[]}>('owner_totp_enable', {otp:otp.value}); otp.value = '';
                await refresh(); showOwnerTab('security');
                const recovery = node('div', '', 'security-block recovery-codes'); recovery.setAttribute('data-editing','true');
                recovery.append(node('h3', 'Guarde seus códigos de recuperação'), node('p', 'Cada código funciona uma vez, junto com sua senha. Eles aparecem somente agora. Guarde fora deste computador para recuperar o acesso se perder o celular.'));
                const codes = node('pre', enabled.recoveryCodes.map(code => code.match(/.{1,4}/g)!.join('-')).join('\n')); recovery.append(codes);
                recovery.append(button('Já guardei os códigos', () => { recovery.remove(); })); element('owner-security').prepend(recovery); message('Duas etapas ativadas. As outras sessões foram encerradas.');
            }, '', error => feedback.replaceChildren(node('p',error,'notice error'))), feedback);
        }, 'secondary', error => setup.replaceChildren(node('p',error,'notice error'))), setup); form.onsubmit = event => event.preventDefault(); mfa.append(form);
    } else {
        const details = node('details'); details.append(node('summary','Gerenciar o autenticador'));
        const form = node('form'); const password = labelledInput(form,'Senha atual do painel','','password'); password.minLength = 14; password.maxLength = 128; password.required = true; password.autocomplete = 'current-password';
        const otp = labelledInput(form,'Código novo do autenticador ou de recuperação'); otp.maxLength = 40; otp.required = true; otp.autocomplete = 'one-time-code'; const feedback = node('div'); feedback.setAttribute('role','status');
        form.append(button('Desativar duas etapas', async () => { if (!form.reportValidity()) return; if (!confirm('Desativar o autenticador? O acesso passará a depender somente da senha.')) return; const secret = password.value; const code = otp.value; password.value = ''; otp.value = ''; await request('owner_totp_disable',{password:secret,otp:code}); message('Autenticador desativado. As outras sessões foram encerradas.'); await refresh(); }, 'danger', error => feedback.replaceChildren(node('p',error,'notice error'))), feedback); form.onsubmit = event => event.preventDefault(); details.append(form); mfa.append(details);
    }
    section.append(mfa);
    const sessions = node('div', '', 'security-block'); sessions.append(node('h3', 'Sessões da central'), node('p', 'Encerre o acesso deste painel em outros navegadores. Sua sessão atual continua aberta.'));
    sessions.append(button('Encerrar outras sessões', async () => { if (!confirm('Encerrar as outras sessões e os links de acesso da central?')) return; await request('owner_revoke_sessions'); message('Outras sessões encerradas.'); await refresh(); }, 'secondary'));
    section.append(sessions);
    const passwordDetails = node('details', '', 'security-block'); passwordDetails.append(node('summary', passwordConfigured ? 'Alterar senha do painel' : 'Criar senha do painel'));
    const passwordForm = node('form');
    const password = labelledInput(passwordForm, 'Nova senha do painel', '', 'password'); password.minLength = 14; password.maxLength = 128; password.autocomplete = 'new-password'; const confirmation = labelledInput(passwordForm, 'Repita a nova senha', '', 'password'); confirmation.maxLength = 128; confirmation.autocomplete = 'new-password';
    const feedback = node('div'); feedback.setAttribute('role','status');
    passwordForm.append(button('Salvar senha do painel', async () => { if (password.value.length < 14 || password.value !== confirmation.value) throw new Error('As senhas precisam ser iguais e ter pelo menos 14 caracteres.'); const secret = password.value; password.value = ''; confirmation.value = ''; await request('owner_password', {password:secret}); message('Senha alterada. As outras sessões foram encerradas.'); await refresh(); }, '', error => feedback.replaceChildren(node('p',error,'notice error'))), feedback); passwordForm.onsubmit = event => event.preventDefault(); passwordDetails.append(passwordForm); section.append(passwordDetails);
}
function renderGrant() {
    const section = element('manual-grant');
    section.replaceChildren(node('h2', 'Atribuir um plano'), node('p', 'Nas compras, o plano vai automaticamente para a conta do pedido. Para atribuições manuais, use o código copiado na aba Amigos do launcher. Nomes iguais podem pertencer a contas diferentes.'));
    const query = labelledInput(section, 'Usuário, código de amizade ou ID completo'); query.maxLength = 64; query.placeholder = 'Ex.: jogador#12ab34cd';
    const results = node('div');
    results.setAttribute('aria-live', 'polite');
    const errorAtResults = (error: string) => { results.replaceChildren(node('p', error, 'notice error')); };
    const selectUser = (user: GrantUser) => {
        const self = user.id === snapshot?.identity.id;
        const form = node('form'); form.setAttribute('data-editing', 'true');
        form.append(node('h3', self ? 'Alterar meu próprio plano' : `Alterar plano de ${user.username}`), node('strong', user.friendCode), node('p', `ID: ${user.id}`, 'order-code'), node('p', `Plano atual: ${planName(user.plan)}${user.plan !== 'free' && user.expires_at ? ` · até ${date(user.expires_at)}` : ''}`));
        if (self) form.append(node('p', 'Esta é a conta autenticada nesta central. Nenhum perfil de mesmo nome será alterado.'));
        const confirmation = self ? null : labelledInput(form, 'Código de amizade enviado pelo jogador');
        if (confirmation) {
            confirmation.maxLength = 64; confirmation.required = true; confirmation.autocomplete = 'off'; confirmation.placeholder = 'Ex.: jogador#12ab34cd';
            const help = node('p', 'Peça à pessoa para abrir Amigos no launcher, copiar Seu código e enviar para você. Cole o código completo aqui para confirmar a conta que receberá o plano.', 'order-code');
            help.id = 'grant-code-help'; confirmation.setAttribute('aria-describedby', help.id); form.append(help);
        }
        const label = node('label', 'Novo plano'); const select = node('select');
        for (const value of ['free', 'plus', 'pro'] as Plan[]) { const option = node('option', planName(value)); option.value = value; select.append(option); }
        select.value = user.plan; label.append(select); form.append(label);
        const days = labelledInput(form, 'Prazo em dias (substitui o vencimento atual)', '30', 'number'); days.min = '1'; days.max = '365'; days.required = true;
        const reason = labelledInput(form, 'Motivo da alteração (opcional)'); reason.maxLength = 240;
        form.append(node('p', 'Uma atribuição manual não registra um pagamento aprovado.'));
        const feedback = node('div'); feedback.setAttribute('role', 'status'); feedback.setAttribute('aria-live', 'polite');
        const showError = (error: string) => { feedback.replaceChildren(node('p', error, 'notice error')); };
        form.append(button('Confirmar atribuição', async () => {
            feedback.replaceChildren();
            if (!form.reportValidity()) return;
            const code = confirmation?.value.trim() || user.friendCode;
            if (confirmation && ![user.friendCode.toLowerCase(), user.id.toLowerCase()].includes(code.toLowerCase())) throw new Error('O código informado não corresponde à conta selecionada. Confira o código enviado pelo jogador.');
            if (!confirm(`Atribuir ${planName(select.value as Plan)} a ${user.friendCode}?\nID: ${user.id}\nPrazo: ${days.value} dias.`)) return;
            const updated = await request<{ identity: { id: string; friendCode: string }; membership: Snapshot['membership'] }>('owner_grant', { targetId: user.id, confirmationCode: code, plan: select.value, days: Number(days.value), revision: user.revision, note: reason.value });
            const success = `${updated.identity.friendCode} · ${planName(updated.membership.plan)}${updated.membership.plan !== 'free' ? ` ativo até ${date(updated.membership.expiresAt!)}` : ''}. Alteração gravada no histórico.`;
            await refresh();
            message(success);
        }, '', showError), button('Voltar à busca', () => { results.replaceChildren(); results.removeAttribute('data-editing'); }), feedback);
        form.onsubmit = event => event.preventDefault(); results.replaceChildren(form);
    };
    section.append(button('Alterar meu próprio plano', async () => {
        results.setAttribute('data-editing', 'true');
        const found = await request<{ users: GrantUser[] }>('owner_find', { query: snapshot!.identity.id });
        const self = found.users.find(user => user.id === snapshot!.identity.id);
        if (!self) throw new Error('Sua conta não foi encontrada. Atualize o painel.');
        selectUser(self);
    }, 'secondary', errorAtResults));
    section.append(button('Buscar conta', async () => {
        results.setAttribute('data-editing', 'true');
        const found = await request<{ users: GrantUser[] }>('owner_find', { query: query.value.trim() }); results.replaceChildren();
        if (!found.users.length) results.append(node('p', 'Nenhuma conta encontrada. A pessoa precisa ter conectado o launcher à rede social.'));
        for (const user of found.users) {
            const row = node('article', '', 'order-card'); row.append(node('strong', `${user.display_name || user.username} · ${planName(user.plan)}`), node('p', user.friendCode), node('p', `ID: ${user.id}`, 'order-code'));
            if (user.id === snapshot?.identity.id) row.append(node('span', 'Sua conta autenticada', 'status-chip'));
            row.append(button(`Selecionar ${user.friendCode}`, () => selectUser(user))); results.append(row);
        }
    }, 'secondary', errorAtResults), results);
}
async function refresh(background = false): Promise<boolean> {
    const generation = ++refreshGeneration;
    const current = () => generation === refreshGeneration && (!background || !busy);
    if (ownerPage) {
        const state = await request<Snapshot>('status'); if (!current()) return false; if (!state.identity.owner) { lock(); throw new Error('Acesso restrito ao dono.'); }
        const queue = await request('owner_queue', { status: element<HTMLSelectElement>('filter').value, offset });
        if (!current()) return false;
        snapshot = state;
        element('identity').hidden = true; element('owner-view').hidden = false; queueTotal = queue.total;
        const account = element('owner-account'); account.hidden = false; account.replaceChildren(node('p', 'SUA CONTA', 'eyebrow'), node('h2', `${state.identity.username} · ${planName(state.membership.plan)}`), node('p', `${state.identity.username}#${state.identity.id.slice(0, 8)}`, 'order-code'));
        element('pending-count').textContent = String(queue.stats.pending || 0);
        element('session-status').textContent = state.security?.expiresAt ? `Sessão até ${new Date(state.security.expiresAt * 1000).toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'})}` : 'Sessão protegida';
        element('stats').replaceChildren(...[['Esperando verificação', queue.stats.pending || 0], ['Pedidos aprovados', queue.stats.approved || 0], ['Valor aprovado', money(queue.stats.approvedAmount || 0)]].map(([title, value]) => { const card = node('div', '', 'stat-card'); card.append(node('span', String(title)), node('strong', String(value))); return card; }));
        syncOrders(queue.orders, true); if (!queue.orders.length) { const empty = node('div','','owner-empty'); empty.append(node('span','✓','empty-mark'),node('h3','Tudo em dia.'),node('p','Nenhum pedido neste filtro. Quando chegar um comprovante, ele aparece aqui.')); element('order-list').append(empty); } pagination(); element('queue-pagination').hidden = queue.total <= 50;
        if (!background) {
        const settings = element('owner-settings'); settings.replaceChildren(node('p', 'Pause novos pedidos antes de alterar o recebimento. Pedidos já criados conservam a chave original.'));
        const form = node('form'); const bank = labelledInput(form, 'Banco', queue.config.bank); bank.readOnly = true; const key = labelledInput(form, 'Chave Pix', queue.config.pixKey); key.maxLength = 100; const label = node('label'); const enabled = node('input'); enabled.type = 'checkbox'; enabled.checked = queue.config.enabled; label.append(enabled, document.createTextNode('Aceitar novos pedidos de planos')); form.append(label, button('Salvar configuração', async () => { await request('owner_config', { bank: bank.value, pixKey: key.value.trim(), enabled: enabled.checked }); message('Configuração salva.'); await refresh(); }, '')); form.onsubmit = event => event.preventDefault(); settings.append(form);
        renderGrant(); renderSecurity(queue.config.passwordConfigured); showOwnerTab(ownerTab);
        }
        element('audit').replaceChildren(...queue.audit.map((entry: any) => { const row = node('article','','owner-audit'); row.append(node('span', ['security','password'].includes(entry.action) ? 'ACESSO' : 'GESTÃO','audit-kind'),node('strong',entry.username ? `${entry.username}${entry.social_id ? `#${entry.social_id.slice(0,8)}` : ''}` : 'Conta'),node('p',entry.detail),node('time',date(entry.created_at))); return row; })); if (!queue.audit.length) element('audit').append(node('p', 'As decisões aparecerão aqui.'));
        updateApprovalButtons();
    } else {
        const state = await request<Snapshot>('status'); if (!current()) return false; snapshot = state; accountResolved = true; renderPlanSummary();
    }
    return true;
}
function scheduleUpdates(delay = 5000) {
    clearTimeout(updatesTimer);
    updatesTimer = setTimeout(() => void pollUpdates(), delay);
}
async function pollUpdates() {
    if (updatesRunning) return;
    if (!snapshot || busy || unlocking || document.hidden) { scheduleUpdates(); return; }
    updatesRunning = true;
    let delay = 5000;
    try {
        const result = await request<{ revision: string }>(ownerPage ? 'owner_updates' : 'updates');
        if (!snapshot || busy || document.hidden) return;
        if (result.revision !== lastUpdateRevision && await refresh(true)) lastUpdateRevision = result.revision;
        const status = document.getElementById('orders-sync');
        if (status) status.textContent = 'Atualização automática ativa · conferindo a cada 5 segundos.';
    } catch {
        delay = 30000;
        const status = document.getElementById('orders-sync');
        if (status) status.textContent = snapshot ? 'Conexão interrompida. Tentaremos atualizar novamente em instantes.' : '';
    } finally { updatesRunning = false; scheduleUpdates(delay); }
}
async function start() {
    const ticket = new URLSearchParams(location.hash.slice(1)).get('ticket');
    if (location.hash) history.replaceState(null, '', location.pathname);
    try { if (ticket) {await request('exchange', { ticket });window.dispatchEvent(new Event('luxmc-session-changed'));} await refresh(); } catch (error) { if (!ownerPage && !ticket && accountResolved && !snapshot) message(); else message((error as Error).message, true); }
    if (ownerPage) element('refresh').onclick = () => void perform(refresh);
    if (ownerPage) {
        for (const control of document.querySelectorAll<HTMLButtonElement>('[data-owner-tab]')) control.onclick = () => { lastInteraction = Date.now(); showOwnerTab(control.dataset.ownerTab!); };
        element<HTMLSelectElement>('filter').onchange = () => { offset = 0; selectedReceipt = ''; void perform(refresh); };
        element('previous').onclick = () => { offset = Math.max(0, offset - 50); selectedReceipt = ''; void perform(refresh); };
        element('next').onclick = () => { offset += 50; selectedReceipt = ''; void perform(refresh); };
        element('logout').onclick = () => void perform(async () => { await request('logout'); lock(); message('Acesso encerrado.'); });
    } else {
        element('close-checkout').onclick = () => element<HTMLDialogElement>('checkout').close();
        element<HTMLDialogElement>('checkout').onclose = () => { if (selectedReceipt) clearReceipt(selectedReceipt); selectedReceipt = ''; if (snapshot) syncOrders(snapshot.orders, false); for (const form of element('checkout').querySelectorAll('[data-editing]')) form.removeAttribute('data-editing'); };
        for (const control of document.querySelectorAll<HTMLButtonElement>('[data-plan]')) control.onclick = () => void perform(async () => {
            if (!snapshot) { location.assign('/conta'); return; }
            const pending = snapshot.orders.find(order => order.plan === control.dataset.plan && ['awaiting_receipt', 'pending'].includes(order.status));
            if (pending) { openCheckout(); return; }
            if (snapshot.membership.plan === control.dataset.plan || !snapshot.checkout.enabled) { updatePlanAvailability(); return; }
            try { await request('create', { plan: control.dataset.plan }); } catch (error) { await refresh(); throw error; }
            await refresh(); openCheckout(); message('Confira o destinatário e o valor antes de pagar.');
        });
    }
    document.addEventListener('input', event => { if (event.target instanceof Element) event.target.closest('form,.order-card')?.setAttribute('data-editing','true'); });
    for (const event of ['pointerdown','keydown','input','scroll']) document.addEventListener(event, () => { lastInteraction = Date.now(); }, {passive:true,capture:true});
    setInterval(() => {
        if (ownerPage && snapshot && !busy && !unlocking && Date.now() - lastInteraction >= 300000) { void perform(async () => { try { await request('logout'); } finally { lock(); message('A central foi bloqueada por inatividade. Entre novamente.'); } }); return; }
        if (ownerPage && snapshot && !busy && !unlocking && !document.hidden && Date.now() - lastHeartbeat >= 60000 && Date.now() - lastInteraction < 60000) { lastHeartbeat = Date.now(); void perform(async () => { await request('owner_touch'); }); return; }
    }, 30000);
    scheduleUpdates();
    document.addEventListener('visibilitychange', () => { if (!document.hidden) scheduleUpdates(0); });
    window.addEventListener('pagehide', () => { clearTimeout(updatesTimer); refreshGeneration++; for (const id of receiptRequests.keys()) clearReceipt(id); for (const id of receiptUrls.keys()) clearReceipt(id); });
    window.addEventListener('pageshow', event => { if (event.persisted) { if (selectedReceipt) { const card = element('order-list').querySelector<HTMLElement>(`[data-order="${selectedReceipt}"]`); if (card) delete card.dataset.signature; syncOrders(visibleOrders, ownerPage); } scheduleUpdates(0); } });
}
void start();

window.addEventListener('luxmc-language-changed', () => { updatePlanAvailability(); if (!ownerPage && snapshot) renderPlanSummary(); });
