# Código do site

As fontes do servidor ficam em `site-server/`; as fontes das páginas e da prévia 3D ficam em `site-client/`. Edite esses arquivos TypeScript. Os arquivos JavaScript correspondentes em `website/` são saídas de compilação e continuam servidos no mesmo endereço.

`pnpm check:website` verifica os dois projetos em modo estrito. `pnpm build:website` verifica os tipos e gera servidor, scripts das páginas e bundles visuais. `pnpm test:website` gera os scripts usados pelos testes antes de executar a suíte.

Os validadores de mídia recebem dados desconhecidos e os validam antes de salvar. Os contratos de requisições, elementos do portal e bindings de banco estão explícitos. Algumas estruturas dinâmicas de banco e respostas externas ainda usam `any`; a migração das fontes não equivale a uma tipagem completa de todos os dados externos.

A publicação dos novos campos de perfil requer a migração `website/migrations/0009_profile_identity.sql`, aplicada antes do deploy. A migração mantém os perfis existentes e usa valores vazios para nome de exibição e status.
