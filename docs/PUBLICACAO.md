# Publicação do Luxmc

O portal e a API social estão preparados para publicação posterior. Nenhum banco remoto foi criado, nenhuma migração remota foi aplicada e nenhum deploy foi executado nesta entrega.

## Validar localmente

Na raiz do repositório:

```sh
pnpm check
pnpm test:run
pnpm test:website
pnpm build
pnpm electron:compile
```

Em `src-tauri/`:

```sh
cargo check
cargo test --lib
```

A chave padrão original do CurseForge está preservada como fallback. As configurações pessoais no keyring e em `CURSEFORGE_API_KEY` continuam disponíveis. O teste de integração usa a API real e exige acesso à rede; pode ser executado isoladamente:

```sh
cargo test --lib test_curseforge_api_search_and_file_details
```

Os testes do website exercitam o SQL das migrações com SQLite, autenticação, consentimento, privacidade, expiração de presença e seleção de downloads. Exigem Node 22.13+ com `node:sqlite`.

## Ativar Cloudflare D1 quando for publicar

Na raiz do repositório, autentique localmente; nunca coloque tokens no código ou em mensagens:

```sh
pnpm dlx wrangler@4.135.0 login
pnpm dlx wrangler@4.135.0 d1 create luxmc-social
```

Acrescente o bloco retornado ao arquivo `website/wrangler.toml`, usando o ID real criado:

```toml
[[d1_databases]]
binding = "SOCIAL_DB"
database_name = "luxmc-social"
database_id = "ID_REAL_RETORNADO_PELO_COMANDO"
migrations_dir = "migrations"
```

O binding deve se chamar exatamente `SOCIAL_DB`. Não publique um ID de exemplo. Configure esse mesmo binding para o ambiente de produção do projeto Pages; use outro banco para previews caso precise testar sem acessar dados reais.

Aplique a migração antes do deploy:

```sh
pnpm dlx wrangler@4.135.0 d1 migrations apply luxmc-social --remote --config website/wrangler.toml
```

Publique o portal:

```sh
pnpm deploy:website
```

O projeto Pages configurado é `luxmc`. O launcher usa `https://luxmc-r92.pages.dev/api/social/` em `src-tauri/src/commands/social.rs`; confirme que é o domínio do projeto de destino. Se mudar, altere a constante no Rust e gere novos binários do launcher. A API não exige um token global embutido: cada instalação cria sua credencial aleatória e o D1 armazena apenas seu hash.

## Conferir a publicação

- `/api/latest-release` deve retornar a versão e os assets do GitHub oficial.
- `/download/linux` e `/download/windows` devem redirecionar para assets da release. Se não houver asset adequado ou o GitHub estiver indisponível, o endpoint deve abrir a página oficial de releases, sem baixar um arquivo de outra plataforma.
- Abra o launcher em duas instalações, conecte a rede social, procure o nickname e confirme também o código do amigo. Envie e aceite um convite; verifique presença, remoção e expiração depois de fechar uma instalação.
- Compartilhe um mundo LAN com UPnP e teste a entrada pela outra máquina. O roteador precisa oferecer UPnP e IP público acessível; CGNAT não é resolvido por esta implementação. Não há relay de tráfego.
- Teste importação e lançamento de um pack CurseForge e de um `.mrpack`, incluindo cancelamento e reparo de arquivo ausente.

Sem `SOCIAL_DB`, a API retorna 503 com mensagem de configuração pendente. A interface mantém o launcher utilizável e informa o erro de conexão.

## Dados e recuperação

Nicknames e skins são identificadores visuais; não constituem autenticação Microsoft. Convites só passam a compartilhar presença e endereço do mundo depois da aceitação pelo destinatário. A presença expira em 75 segundos; o launcher consulta a cada 25 segundos. Credenciais permanecem no diretório de dados nativo `social/` e nunca são retornadas ao frontend. Não remova esse diretório se quiser manter a identidade social da instalação.

A atualização de modpacks prepara os arquivos em outra pasta, valida downloads e preserva dados pessoais antes de trocar o caminho da instância em uma transação SQLite. A pasta anterior permanece no disco como backup. Não a remova antes de testar o novo pack. Alterações manuais com o jogo aberto devem ser evitadas.

Antes de aplicar futuras migrações em produção, exporte o D1:

```sh
pnpm dlx wrangler@4.135.0 d1 export luxmc-social --remote --output /tmp/luxmc-social-backup.sql --config website/wrangler.toml
```

O backup contém dados de usuários e hashes de credenciais: mantenha-o fora do repositório. Para rollback do frontend/Functions, use uma implantação anterior do Pages. Restaurar o código não reverte migrações do banco.


## Testes locais completos antes de publicar

Execute:

```sh
cargo build --manifest-path src-tauri/Cargo.toml
node tests/native-smoke.mjs --launch
```

O teste nativo é para Linux, usa perfis e banco exclusivos em `/tmp`, baixa o Fabulously Optimized 5.4.1 publicado em ambos os provedores, verifica hashes, arquivos desativados, reparo, importação aninhada, cancelamento e inicialização do Minecraft. Fecha somente os processos de jogo que ele próprio iniciou. Precisa de rede e sessão gráfica; os logs ficam na pasta temporária informada.

Para o serviço social, `tests/social-http.mjs` aceita apenas localhost. A configuração `website/wrangler.local.toml` é exclusivamente de teste; seu identificador não é um banco remoto. Prepare uma cópia temporária do website, substitua seu `wrangler.toml` por essa configuração, aplique a migração com `d1 migrations apply luxmc-social-local --local --config website/wrangler.local.toml --persist-to /tmp/luxmc-d1-smoke`, e execute Pages dev nessa cópia usando a mesma pasta de persistência. Depois:

```sh
node tests/social-http.mjs http://127.0.0.1:8795
```

Pages não aceita `--config` personalizado. O script `pnpm deploy:website` entra no diretório `website` com `--cwd` e usa automaticamente `website/wrangler.toml`. Nenhum identificador de teste deve ser copiado para a configuração de produção.

O workflow CI agora valida frontend, website, Rust e build em Linux e Windows, com lockfiles congelados. Foi preparado, mas não foi disparado remotamente nesta sessão.
