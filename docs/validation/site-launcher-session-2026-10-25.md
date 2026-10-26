# Sessão do launcher no site

A página pode reconhecer a conta ativa no launcher por uma conexão HTTP em `127.0.0.1:38457`. O serviço escuta somente no loopback, permite apenas as três origens oficiais e exige POST JSON com cabeçalho próprio. GET, origem desconhecida, Host alterado, cabeçalhos duplicados e conteúdo acima do limite são rejeitados. Há limite de conexões, tempo e frequência.

A resposta contém um ticket de uso único, válido por 60 segundos. O site troca esse ticket por cookies HttpOnly, Secure e SameSite=Strict. A sessão completa da conta só é aberta se o perfil já estiver associado a uma conta cadastrada. A área do dono continua exigindo sua confirmação própria. Nenhum token Microsoft ou Minecraft é transferido ao navegador.

O botão Portal Luxmc usa o mesmo mecanismo pelo fragmento do endereço. O script remove o ticket do histórico antes da troca. Isso também oferece um caminho quando o navegador bloqueia conexões locais. O acesso direto ao site pode exigir a permissão de rede local do navegador; essa permissão não é contornada.

Sair do site guarda a escolha de não reconectar automaticamente. A página Conta oferece uma reconexão explícita. Uma sessão já aberta pelo usuário no site é preservada ao visitar o endereço diretamente; abrir o Portal pelo launcher transfere explicitamente a conta escolhida nele.

## Verificação local

- Svelte/TypeScript: zero erros e zero avisos.
- Frontend: 230 testes aprovados e compilação de produção concluída.
- Backend: `cargo check` aprovado; 292 testes aprovados e 12 testes dependentes de fixtures externas ignorados.
- Site: 142 testes aprovados; compilação TypeScript e geração dos arquivos concluídas.
- Testes de transferência verificam uso único, cookies privados, conta cadastrada, abertura pelo fragmento, descoberta local e respeito ao logout.

Os testes usam identidades simuladas. O agente não entrou na conta Microsoft pessoal nem iniciou Minecraft. A revisão exige instalar o launcher correspondente para ativar o conector local; apenas atualizar o site não acrescenta esse serviço a um executável antigo.
