# Verificação — Luxmc 3.9.0

A release [Luxmc 3.9 — Sua Marca, Seu Mundo](https://github.com/predabr/luxmc/releases/tag/v3.9.0) foi publicada em 12 de outubro de 2026 (UTC), após a conclusão do [workflow Release](https://github.com/predabr/luxmc/actions/runs/38188692669). A validação das fontes, as compilações Windows/Linux/macOS, os testes Rust, o ciclo de instalação no Windows e a conferência dos sete instaladores com manifestos Ed25519 assinados passaram.

O download público do site foi conferido depois da publicação. Os arquivos do GitHub responderam, as assinaturas dos manifestos foram verificadas e os bytes do instalador servido pelo site corresponderam ao hash publicado: `d18dd7c81cfc174982a915b65465e0b9af7bed8cca27ab013db2797845ef0e2d`, 21.433.749 bytes.

## Confiança Windows

O instalador anterior 3.8.0 foi conferido em 11 de outubro de 2026: SHA-256 `e7c1ed8b3ec9971bb32c9068d66a7db9cee1d793c1e6b6a27f7c9c2cac19f017`, 21.371.759 bytes, Authenticode `NotSigned`. A análise personalizada do Microsoft Defender retornou zero e não encontrou ameaças, usando definições `1.459.670.0`. Esse resultado pertence ao arquivo 3.8 e não comprova a segurança da versão 3.9.

O workflow 3.9 produziu `windows-release-audit.json`, com hashes do instalador, programa e auxiliar de recuperação. Os três executáveis continuam com Authenticode `NotSigned`. A análise personalizada do Defender retornou zero e não encontrou ameaças nos três componentes, usando definições `1.459.676.0`. O artefato de validação pertence à execução vinculada à tag. Nenhuma alteração na proteção do Windows foi necessária para essa conferência.

O instalador 3.9 foi enviado ao VirusTotal com autorização para os termos e o compartilhamento do artefato público. O [relatório de 12 de outubro de 2026 (UTC)](https://www.virustotal.com/gui/file/d18dd7c81cfc174982a915b65465e0b9af7bed8cca27ab013db2797845ef0e2d/detection) registrou **3/70 alertas**: Avast e AVG (`NSIS:MalwareX-gen [Misc]`) e Trapmine (`Suspicious.low.ml.score`). ClamAV teve timeout e alguns mecanismos não processaram esse tipo de arquivo. Não houve confirmação independente de falso positivo.

Os pedidos de revisão do instalador anterior 3.8 foram recebidos pela Avast, AVG e Microsoft. A resposta dessas análises continua pendente. O relatório do instalador 3.9 não deve ser confundido com a revisão do arquivo 3.8.

Os formulários de revisão do arquivo 3.9 também tiveram o recebimento confirmado pela Avast, AVG e Microsoft, após a verificação humana dos CAPTCHAs. O pedido à Trapmine foi enviado por e-mail ao contato oficial indicado pelo fornecedor. Nenhum desses envios equivale a uma aprovação; as respostas dos analistas continuam pendentes. Somente o instalador público e os dados da compilação foram compartilhados, sem mundos, tokens ou instâncias de jogadores.

As detecções Avast, AVG e Trapmine informadas pelo usuário não foram classificadas como falsos positivos confirmados. A candidatura gratuita à SignPath Foundation foi recusada. Esta release permanece sem certificado público de assinatura de código e pode ser bloqueada pelo Windows. Revisão de fornecedores e assinatura são processos diferentes.

## Personalização e desempenho

O relatório local de benefícios registra `pnpm check`, build de produção, 75 testes de API e nove cenários de navegador Free/Plus/Pro, além da regressão de imagens e GIFs. Os testes de navegador usam contas simuladas e não modificam contas pessoais.

Na preparação 3.9, os 160 testes unitários do frontend, 75 testes das APIs e 15 testes de dependência/manifestos passaram. As 20 suítes de navegação foram conferidas, com repetição das oito últimas depois da correção de movimento reduzido nos cartões. O teste de hover agora verifica um deslocamento pequeno em vez de exigir exatamente três pixels. A compilação nativa local continua bloqueada pela política de execução do Windows; as máquinas do GitHub Actions fazem a validação e geração dos pacotes antes da publicação.

O cache do Shield continua identificado pelo hash do arquivo. A auditoria de metadados para instalação usa leitura limitada; a análise de segurança antes da abertura continua obrigatória. Testes Rust verificam alterações de arquivos, CRC, cancelamento e a invalidação dos caches.

A auditoria npm identificou [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm), sem versão upstream corrigida na preparação. O lockfile aplica `patches/braces@3.0.3.patch`, limitando a profundidade a 128 na análise e nos percursos recursivos. `tests/braces-depth.test.mjs` verifica padrões normais, texto profundamente aninhado e chamadas AST diretas. O aviso continua visível na auditoria npm porque a versão do pacote não muda; não foi ocultado por uma exceção de auditoria. O pacote é uma dependência das ferramentas de glob usadas na compilação Node, não um programa executado pelo instalador Windows.

A publicação das novas funções do site requer a migração aditiva `0016_profile_style.sql`. Planos, aprovações de Pix e preferências existentes devem ser preservados; a nova release não ativa compras pausadas nem provisiona servidores.

A migração `0016_profile_style.sql` foi aplicada ao D1 de produção durante a preparação desta release, sem apagar perfis. Uma consulta agregada posterior conferiu que o campo está disponível nos perfis existentes. A publicação das funções do portal acompanha a entrega dos novos artefatos.

## Conta e planos no site

O portal publicado mostra a identidade conectada ao lado do download: a foto do perfil do launcher ou a cabeça da skin publicada pela mesma conta. A sessão usa cookies HttpOnly e uma API limitada à identidade autenticada. A seleção de Plus ou Pro cria o pedido para essa identidade; não depende de digitar um nome. A conferência do Pix permanece manual e pedidos pausados não foram ativados.

A página de planos conserva o nome, plano e vencimento. A lista de pedidos e o editor de aparência saíram da comparação; o comprovante de um pedido existente fica em uma janela própria. O plano já ativo não pode gerar uma compra duplicada. Trocas entre sessão web, ticket do launcher e central do dono eliminam a sessão anterior que poderia selecionar outra conta. Sair do site revoga somente as sessões desse navegador, sem apagar perfis ou desconectar os outros dispositivos.

Os 83 testes das APIs e regras do site passaram, incluindo troca de identidade, autorização da imagem do perfil, logout com CSRF e seleção de plano. Os testes de navegador verificaram foto, recorte da cabeça da skin, plano ativo, pedidos pausados, comprovante e o cabeçalho em desktop e celular. Os arquivos JavaScript e CSS publicados foram comparados pelos hashes com os arquivos locais. Os testes de navegador de contas e pagamentos usaram fixtures, sem aprovar compras reais ou alterar planos pessoais.

A visita à página sem sessão exibe a orientação para conectar a conta, sem apresentar o login esperado como erro. Uma falha real de serviço continua visível. Esses dois estados foram conferidos no navegador; o teste de nomes iguais aguarda a resposta da busca antes de conferir as duas contas e conserva o bloqueio de atribuição com código incorreto.

## Limites

Os testes automatizados não equivalem a uma avaliação completa de malware nem garantem compatibilidade com todos os modpacks. O teste P2P entre duas redes no One Terror Block 1.20.1 permanece pendente dos participantes. O launcher instalado do usuário não foi iniciado para preparar esta release.
