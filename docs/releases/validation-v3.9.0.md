# Verificação — Luxmc 3.9.0

A release deve ser publicada somente depois que o fluxo Release concluir a validação das fontes, as compilações Windows/Linux/macOS, os testes Rust, o ciclo de instalação no Windows e a conferência dos instaladores com manifestos Ed25519 assinados.

## Confiança Windows

O instalador anterior 3.8.0 foi conferido em 11 de outubro de 2026: SHA-256 `e7c1ed8b3ec9971bb32c9068d66a7db9cee1d793c1e6b6a27f7c9c2cac19f017`, 21.371.759 bytes, Authenticode `NotSigned`. A análise personalizada do Microsoft Defender retornou zero e não encontrou ameaças, usando definições `1.459.670.0`. Esse resultado pertence ao arquivo 3.8 e não comprova a segurança da versão 3.9.

O workflow 3.9 produz `windows-release-audit.json`, com hashes do instalador, programa e auxiliar de recuperação, estado Authenticode e análise personalizada do Defender. O artefato de validação pertence à execução vinculada à tag. Nenhuma alteração na proteção do Windows é necessária para essa conferência.

As detecções Avast, AVG e Trapmine informadas pelo usuário não foram classificadas como falsos positivos confirmados. A candidatura gratuita à SignPath Foundation foi recusada. Esta release permanece sem certificado público de assinatura de código e pode ser bloqueada pelo Windows. Revisão de fornecedores e assinatura são processos diferentes.

## Personalização e desempenho

O relatório local de benefícios registra `pnpm check`, build de produção, 75 testes de API e nove cenários de navegador Free/Plus/Pro, além da regressão de imagens e GIFs. Os testes de navegador usam contas simuladas e não modificam contas pessoais.

Na preparação 3.9, os 160 testes unitários do frontend, 75 testes das APIs e 15 testes de dependência/manifestos passaram. As 20 suítes de navegação foram conferidas, com repetição das oito últimas depois da correção de movimento reduzido nos cartões. O teste de hover agora verifica um deslocamento pequeno em vez de exigir exatamente três pixels. A compilação nativa local continua bloqueada pela política de execução do Windows; as máquinas do GitHub Actions fazem a validação e geração dos pacotes antes da publicação.

O cache do Shield continua identificado pelo hash do arquivo. A auditoria de metadados para instalação usa leitura limitada; a análise de segurança antes da abertura continua obrigatória. Testes Rust verificam alterações de arquivos, CRC, cancelamento e a invalidação dos caches.

A auditoria npm identificou [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm), sem versão upstream corrigida na preparação. O lockfile aplica `patches/braces@3.0.3.patch`, limitando a profundidade a 128 na análise e nos percursos recursivos. `tests/braces-depth.test.mjs` verifica padrões normais, texto profundamente aninhado e chamadas AST diretas. O aviso continua visível na auditoria npm porque a versão do pacote não muda; não foi ocultado por uma exceção de auditoria. O pacote é uma dependência das ferramentas de glob usadas na compilação Node, não um programa executado pelo instalador Windows.

A publicação das novas funções do site requer a migração aditiva `0016_profile_style.sql`. Planos, aprovações de Pix e preferências existentes devem ser preservados; a nova release não ativa compras pausadas nem provisiona servidores.

A migração `0016_profile_style.sql` foi aplicada ao D1 de produção durante a preparação desta release, sem apagar perfis. Uma consulta agregada posterior conferiu que o campo está disponível nos perfis existentes. A publicação das funções do portal acompanha a entrega dos novos artefatos.

## Limites

Os testes automatizados não equivalem a uma avaliação completa de malware nem garantem compatibilidade com todos os modpacks. O teste P2P entre duas redes no One Terror Block 1.20.1 permanece pendente dos participantes. O launcher instalado do usuário não foi iniciado para preparar esta release.
