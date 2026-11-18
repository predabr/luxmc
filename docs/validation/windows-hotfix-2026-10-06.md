# Luxmc 3.0.2 — correções locais e entrega pelo site

Não houve alteração de versão, commit, tag ou publicação no GitHub nesta entrega.

## Correções

- Aplicação de aparência resolve a conta salva pelo ID e pelo UUID, incluindo UUID com hífens ou letras maiúsculas. Um cadastro Microsoft ausente só é restaurado após validar o token e a identidade com o serviço oficial; uma sessão inválida exige novo login.
- Mundos LAN de convidados são anunciados na sala e encaminhados tanto para o criador quanto para outros convidados. Fechar um mundo retira seu anúncio; trocar de porta não exige recriar a sala. Convites renovados preservam as conexões já admitidas.
- A adaptação de login atua no servidor integrado do jogo, somente para conexões loopback na porta LAN descoberta enquanto a sala está ativa. O controle expira após quatro segundos sem o launcher. Não altera autenticação de servidores públicos nem concede licença Microsoft ou acesso ao Realms.
- Perfis oficiais em LAN com UUID offline podem recuperar a aparência pública nos serviços oficiais. A consulta aceita apenas o domínio de texturas da Mojang, limita o tamanho da resposta e mantém o resultado em cache.
- Fechar a janela mantém o launcher no ícone da bandeja. Abrir restaura a janela e Sair encerra o processo. A opção pode ser desativada nas configurações. A persistência preserva a conta ativa e outras configurações nativas.
- Falhas da consulta de arquivos do CurseForge são diferenciadas de incompatibilidades reais. Farmer's Delight Cutting Compat não oferece arquivos para 26.3; a consulta real encontrou dois arquivos para 1.21.1.
- O cadastro do site consulta nomes oficiais e sugere alternativas verificadas. As rotas oficiais são prioritárias; PlayerDB fornece um fallback público quando a hospedagem é recusada pelo serviço oficial. Indisponibilidade não é tratada como nome livre.

## Validação

- `pnpm check`: zero erros e avisos.
- Vitest: 19 arquivos, 122 testes aprovados.
- Testes de site: 33 testes aprovados, mais o teste específico do fallback de nomes.
- `cargo check`: aprovado; quatro avisos preexistentes.
- Nove testes de túnel aprovados, incluindo roteamento entre convidados, troca de porta, retirada do anúncio, convites, limites e encerramento. O teste adicional com relay público também passou.
- Resolução de identidade da aparência e preservação da conta ao atualizar configurações: aprovadas.
- Agente Java: isolamento por endereço loopback, porta e expiração verificado. Classes reais de login de 1.8.9, 1.20.1 e 26.3 carregadas com Java 8, 17 e 25, respectivamente. Consulta de skin oficial também aprovada.
- Browser: importação e aplicação de skins, reaplicação sem alterar seleção, sincronização Microsoft com falha e nova tentativa, e instalação dos quatro tipos de conteúdo aprovadas.
- Windows: janela fechada durante o teste e processo mantido; reabertura restaurou a mesma janela e o mesmo PID.

O teste de entrada no mundo entre os computadores do usuário e do primo continua necessário para confirmar a combinação específica de rede, versão e mods. Os testes automatizados não demonstram funcionamento em todas as redes e modpacks.

O instalador mantém `Lux MC Launcher.exe` e a versão 3.0.2. Como não houve nova versão no GitHub, os dois participantes devem instalar o arquivo atualizado do site para usar estas correções.
