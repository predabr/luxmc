import QRCode from 'qrcode';
import type { ProfileStyle } from '../site-server/lib/profile-style';

type Plan = 'free' | 'plus' | 'pro';
interface Preset { id: string; name: string; preferences: Record<string, string | number | boolean> }
interface Order { id: string; socialId: string; username: string; displayName: string; plan: 'plus' | 'pro'; amount: number; bank: string; pixKey: string; days: number; status: string; version: number; createdAt: number; updatedAt: number; note: string; receiptHash: string | null; receiptMime: string | null; receiptUploadedAt: number | null }
interface Snapshot { profile?: {profileStyle:ProfileStyle;displayName:string;status:string}; identity: { id: string; username: string; owner: boolean }; csrfToken?: string | null; security?: { twoFactor: boolean; verifiedUntil: number; expiresAt: number } | null; membership: { plan: Plan; expiresAt: number | null; collections: number; presets: number }; checkout: { enabled: boolean; bank: string; pixKey: string }; settings: { preferences: Record<string, string | number | boolean>; presets: Preset[]; revision: number }; orders: Order[] }
interface GrantUser { id: string; username: string; display_name: string; friendCode: string; plan: Plan; expires_at: number | null; revision: number }
const ownerPage = document.body.dataset.commerce === 'owner';
const element = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const money = (amount: number) => (amount / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
const date = (seconds: number) => new Date(seconds * 1000).toLocaleString('pt-BR');
const planName = (plan: Plan) => `Luxmc ${plan === 'free' ? 'Free' : plan === 'plus' ? 'Plus' : 'Pro'}`;
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
        control.textContent = !accountResolved ? 'Carregando…' : pending ? 'Ver pedido' : current ? 'Seu plano ativo' : !snapshot ? 'Entrar para escolher' : !snapshot.checkout.enabled ? 'Novos pedidos pausados' : `Escolher ${control.dataset.plan === 'plus' ? 'Plus' : 'Pro'} ↗`;
        control.title = current && !pending ? 'Você já tem este plano ativo' : pending ? 'Abrir seu pedido em andamento' : control.textContent;
    }
}
async function perform(action: () => void | Promise<void>, onError?: (error: string) => void) {
    if (busy) return; busy = true; document.querySelectorAll<HTMLButtonElement>('button').forEach(control => control.disabled = true);
    try { await action(); } catch (error) { const text = String((error as Error).message); if (onError) onError(text); else message(text, true); }
    finally { busy = false; document.querySelectorAll<HTMLButtonElement>('button').forEach(control => control.disabled = false); updatePlanAvailability(); pagination(); }
}
function lock() {
    snapshot = null;
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
function renderOrder(order: Order, owner: boolean) {
    const card = node('article', '', 'order-card'); card.dataset.order = order.id;
    const top = node('div', '', 'order-top'); const heading = node('div'); heading.append(node('h3', `${planName(order.plan)} · ${money(order.amount)}`));
    if (owner) heading.append(node('p', order.displayName ? `${order.displayName} (@${order.username})` : order.username), node('p', `Conta: ${order.username}#${order.socialId.slice(0, 8)} · ${order.socialId}`, 'order-code'));
    heading.append(node('p', `Pedido ${order.id}`, 'order-code')); top.append(heading, node('span', statuses[order.status] || order.status, `status-chip ${order.status}`)); card.append(top, node('p', `Criado em ${date(order.createdAt)} · ${order.days} dias a partir da aprovação`));
    if (order.note) card.append(node('p', order.note));
    if (!owner && ['awaiting_receipt', 'pending'].includes(order.status) && snapshot) {
        const pay = node('div', '', 'checkout-block'); pay.append(node('strong', `Pix · ${order.bank} · ${money(order.amount)}`), node('p', 'Confira o destinatário no seu banco antes de pagar. Copiar a chave não faz a transferência.'), node('p', order.pixKey, 'pix-key'));
        pay.append(button('Copiar chave Pix', async () => { await navigator.clipboard.writeText(order.pixKey); message('Chave copiada. Faça a transferência pelo aplicativo do seu banco.'); }));
        const input = labelledInput(pay, order.receiptHash ? 'Substituir comprovante (PNG, JPEG, WebP ou PDF · até 1 MB)' : 'Enviar comprovante (PNG, JPEG, WebP ou PDF · até 1 MB)', '', 'file'); input.accept = 'image/png,image/jpeg,image/webp,application/pdf';
        pay.append(button('Enviar para verificação', async () => {
            const file = input.files?.[0]; if (!file) throw new Error('Escolha o comprovante.'); if (file.size > 1048576) throw new Error('O comprovante precisa ter até 1 MB.');
            const response = await fetch(`/api/receipts/${order.id}`, { method: 'POST', credentials: 'same-origin', headers: { ...headers(), 'Content-Type': file.type }, body: file, signal: AbortSignal.timeout(30000) });
            const result = await response.json(); if (!response.ok) throw new Error(result.error);
            message('Comprovante enviado. Seu plano será liberado após a conferência do crédito no banco.'); await refresh();
        }, ''));
        pay.append(button('Cancelar pedido', async () => { if (!confirm('Cancelar este pedido? Se você já pagou, procure o atendimento antes de cancelar.')) return; await request('cancel', { id: order.id }); await refresh(); }, 'danger')); card.append(pay);
    }
    if (order.receiptHash) card.append(button(selectedReceipt === order.id ? 'Fechar comprovante' : 'Ver comprovante', async () => { selectedReceipt = selectedReceipt === order.id ? '' : order.id; await refresh(); }));
    if (selectedReceipt === order.id && order.receiptHash) {
        const preview = node('div', '', 'receipt-preview');
        if (order.receiptMime === 'application/pdf') { const download = node('a', 'Baixar comprovante PDF', 'commerce-button secondary'); download.href = `/api/receipts/${order.id}`; download.download = 'comprovante.pdf'; preview.append(download); }
        else { const image = node('img'); image.src = `/api/receipts/${order.id}?v=${order.version}`; image.alt = `Comprovante do pedido ${order.id}`; image.className = 'receipt-image'; preview.append(image); }
        preview.append(node('p', `Enviado em ${date(order.receiptUploadedAt!)}. Visível apenas para o comprador e o dono.`));
        if (owner && order.status === 'pending') {
            const form = node('form'); form.append(node('strong', 'Confira o crédito no Nubank antes de aprovar.'));
            const transaction = labelledInput(form, 'Identificador da transação Pix (EndToEndId)'); transaction.maxLength = 64; transaction.autocomplete = 'off';
            const note = labelledInput(form, 'Observação para o comprador / motivo da rejeição'); note.maxLength = 240;
            const label = node('label'); const checked = node('input'); checked.type = 'checkbox'; label.append(checked, document.createTextNode('Conferi o valor e o crédito desta transação no extrato do banco.')); form.append(label);
            const actions = node('div', '', 'toolbar');
            const review = async (decision: string) => { if (decision === 'approve' && !checked.checked) throw new Error('Confira o crédito no banco antes de aprovar.'); const accountCode = `${order.username}#${order.socialId.slice(0, 8)}`; if (!confirm(decision === 'approve' ? `Liberar ${planName(order.plan)} para ${accountCode}?` : `Rejeitar este pedido de ${accountCode}?`)) return; await request('owner_review', { id: order.id, version: order.version, receiptHash: order.receiptHash, decision, transactionId: transaction.value.trim(), creditConfirmed: checked.checked, note: note.value }); selectedReceipt = ''; message(decision === 'approve' ? 'Plano liberado e disponível para sincronização no launcher.' : 'Pedido rejeitado. O motivo está disponível para o comprador.'); await refresh(); };
            actions.append(button('Aprovar e liberar plano', () => review('approve'), ''), button('Rejeitar pedido', () => review('reject'), 'danger')); form.append(actions); form.onsubmit = event => event.preventDefault(); preview.append(form);
        }
        card.append(preview);
    }
    return card;
}
function renderPlanSummary() {
    if (!snapshot) return;
    const identity = element('identity'); identity.hidden = false;
    const details = node('div'); details.append(node('p', 'SEU PLANO', 'eyebrow'), node('h2', `${snapshot.identity.username} · ${planName(snapshot.membership.plan)}`));
    details.append(node('p', snapshot.membership.plan === 'free' ? 'Seu launcher completo, sempre com você.' : `Ativo até ${date(snapshot.membership.expiresAt!)}`));
    const badge = node('span', snapshot.membership.plan === 'free' ? 'Free' : 'Assinatura ativa', 'membership-active');
    identity.replaceChildren(details, badge);
    const availability = element('checkout-availability'); availability.hidden = snapshot.checkout.enabled;
    availability.textContent = 'Novos pedidos estão pausados. Seu plano ativo continua válido.';
    for (const card of document.querySelectorAll<HTMLElement>('[data-tier]')) {
        const current = card.dataset.tier === snapshot.membership.plan;
        card.dataset.current = String(current);
        card.querySelector<HTMLElement>('.plan-current')!.hidden = !current;
    }
    element('order-list').replaceChildren(...snapshot.orders.map(order => renderOrder(order, false)));
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
    if (tab !== 'orders') selectedReceipt = '';
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
        const confirmation = self ? null : labelledInput(form, 'Cole o código que o jogador enviou pelo launcher');
        if (confirmation) { confirmation.maxLength = 64; confirmation.required = true; confirmation.autocomplete = 'off'; }
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
async function refresh() {
    if (ownerPage) {
        const state = await request<Snapshot>('status'); if (!state.identity.owner) { lock(); throw new Error('Acesso restrito ao dono.'); } snapshot = state;
        const queue = await request('owner_queue', { status: element<HTMLSelectElement>('filter').value, offset });
        element('identity').hidden = true; element('owner-view').hidden = false; queueTotal = queue.total;
        const account = element('owner-account'); account.hidden = false; account.replaceChildren(node('p', 'SUA CONTA', 'eyebrow'), node('h2', `${state.identity.username} · ${planName(state.membership.plan)}`), node('p', `${state.identity.username}#${state.identity.id.slice(0, 8)}`, 'order-code'));
        element('pending-count').textContent = String(queue.stats.pending || 0);
        element('session-status').textContent = state.security?.expiresAt ? `Sessão até ${new Date(state.security.expiresAt * 1000).toLocaleTimeString('pt-BR',{hour:'2-digit',minute:'2-digit'})}` : 'Sessão protegida';
        element('stats').replaceChildren(...[['Esperando verificação', queue.stats.pending || 0], ['Pedidos aprovados', queue.stats.approved || 0], ['Valor aprovado', money(queue.stats.approvedAmount || 0)]].map(([title, value]) => { const card = node('div', '', 'stat-card'); card.append(node('span', String(title)), node('strong', String(value))); return card; }));
        element('order-list').replaceChildren(...queue.orders.map((order: Order) => renderOrder(order, true))); if (!queue.orders.length) { const empty = node('div','','owner-empty'); empty.append(node('span','✓','empty-mark'),node('h3','Tudo em dia.'),node('p','Nenhum pedido neste filtro. Quando chegar um comprovante, ele aparece aqui.')); element('order-list').append(empty); } pagination(); element('queue-pagination').hidden = queue.total <= 50;
        const settings = element('owner-settings'); settings.replaceChildren(node('p', 'Pause novos pedidos antes de alterar o recebimento. Pedidos já criados conservam a chave original.'));
        const form = node('form'); const bank = labelledInput(form, 'Banco', queue.config.bank); bank.readOnly = true; const key = labelledInput(form, 'Chave Pix', queue.config.pixKey); key.maxLength = 100; const label = node('label'); const enabled = node('input'); enabled.type = 'checkbox'; enabled.checked = queue.config.enabled; label.append(enabled, document.createTextNode('Aceitar novos pedidos de planos')); form.append(label, button('Salvar configuração', async () => { await request('owner_config', { bank: bank.value, pixKey: key.value.trim(), enabled: enabled.checked }); message('Configuração salva.'); await refresh(); }, '')); form.onsubmit = event => event.preventDefault(); settings.append(form);
        element('audit').replaceChildren(...queue.audit.map((entry: any) => { const row = node('article','','owner-audit'); row.append(node('span', ['security','password'].includes(entry.action) ? 'ACESSO' : 'GESTÃO','audit-kind'),node('strong',entry.username ? `${entry.username}${entry.social_id ? `#${entry.social_id.slice(0,8)}` : ''}` : 'Conta'),node('p',entry.detail),node('time',date(entry.created_at))); return row; })); if (!queue.audit.length) element('audit').append(node('p', 'As decisões aparecerão aqui.'));
        renderGrant(); renderSecurity(queue.config.passwordConfigured); showOwnerTab(ownerTab);
    } else {
        snapshot = await request<Snapshot>('status'); accountResolved = true; renderPlanSummary();
    }
}
async function start() {
    const ticket = new URLSearchParams(location.hash.slice(1)).get('ticket');
    if (location.hash) history.replaceState(null, '', location.pathname);
    try { if (ticket) await request('exchange', { ticket }); await refresh(); } catch (error) { message((error as Error).message, true); }
    if (ownerPage) element('refresh').onclick = () => void perform(refresh);
    if (ownerPage) {
        for (const control of document.querySelectorAll<HTMLButtonElement>('[data-owner-tab]')) control.onclick = () => { lastInteraction = Date.now(); showOwnerTab(control.dataset.ownerTab!); };
        element<HTMLSelectElement>('filter').onchange = () => { offset = 0; selectedReceipt = ''; void perform(refresh); };
        element('previous').onclick = () => { offset = Math.max(0, offset - 50); selectedReceipt = ''; void perform(refresh); };
        element('next').onclick = () => { offset += 50; selectedReceipt = ''; void perform(refresh); };
        element('logout').onclick = () => void perform(async () => { await request('logout'); lock(); message('Acesso encerrado.'); });
    } else {
        element('close-checkout').onclick = () => element<HTMLDialogElement>('checkout').close();
        element<HTMLDialogElement>('checkout').onclose = () => { selectedReceipt = ''; for (const form of element('checkout').querySelectorAll('[data-editing]')) form.removeAttribute('data-editing'); };
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
        if (snapshot && !busy && !document.hidden && !selectedReceipt && !document.querySelector('input:focus,select:focus,textarea:focus,[data-editing]')) void perform(refresh);
    }, 30000);
}
void start();
