const fs = require('node:fs');
const merge = (target, source) => {
  for (const [key, value] of Object.entries(source)) {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      target[key] ??= {};
      merge(target[key], value);
    } else target[key] = value;
  }
};
const skins = JSON.parse(fs.readFileSync('docs/validation/skins-studio-translations.json', 'utf8'));
const settings = JSON.parse(fs.readFileSync('docs/validation/settings-design-translations.json', 'utf8'));
const design = {
  'pt-BR': {
    expandMenu: 'Expandir menu', collapseMenu: 'Recolher menu', webPortal: 'Portal Luxmc',
    friendsDescription: 'Encontre sua turma, acompanhe os convites e abra uma sala para jogar juntos.',
    emptyFriends: 'Sua turma começa com um convite',
    emptyFriendsHint: 'Use Adicionar amigo para buscar um jogador pelo nome ou código. Os pedidos recebidos ficam em Solicitações.',
    loginDescription: 'Escolha sua conta e prepare sua próxima aventura.',
    browserLogin: 'Conclua o login no navegador',
    browserLoginHint: 'Volte a esta janela após autorizar sua conta Microsoft. Sua senha é informada apenas à Microsoft.'
  },
  en: {
    expandMenu: 'Expand menu', collapseMenu: 'Collapse menu', webPortal: 'Luxmc portal',
    friendsDescription: 'Find your friends, manage invitations and open a room to play together.',
    emptyFriends: 'Your group starts with an invitation',
    emptyFriendsHint: 'Use Add friend to find a player by name or code. Incoming requests appear in Requests.',
    loginDescription: 'Choose your account and get ready for your next adventure.',
    browserLogin: 'Finish signing in in your browser',
    browserLoginHint: 'Return to this window after authorizing your Microsoft account. Your password is only entered with Microsoft.'
  },
  es: {
    expandMenu: 'Expandir menú', collapseMenu: 'Contraer menú', webPortal: 'Portal Luxmc',
    friendsDescription: 'Encuentra a tus amigos, gestiona las invitaciones y abre una sala para jugar juntos.',
    emptyFriends: 'Tu grupo empieza con una invitación',
    emptyFriendsHint: 'Usa Añadir amigo para buscar un jugador por nombre o código. Las solicitudes recibidas aparecen en Solicitudes.',
    loginDescription: 'Elige tu cuenta y prepara tu próxima aventura.',
    browserLogin: 'Completa el inicio de sesión en el navegador',
    browserLoginHint: 'Vuelve a esta ventana después de autorizar tu cuenta Microsoft. Tu contraseña se introduce únicamente en Microsoft.'
  }
};
for (const locale of Object.keys(design)) {
  const path = `src/lib/i18n/${locale}.json`;
  const data = JSON.parse(fs.readFileSync(path, 'utf8'));
  merge(data, settings[locale]);
  merge(data, { skinsStudio: skins[locale], design: design[locale] });
  fs.writeFileSync(path, JSON.stringify(data, null, 2) + '\n');
}
console.log('Redesign translations merged for Portuguese, English and Spanish.');
