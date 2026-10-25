# Preparação da licença não comercial do Luxmc

Status em 25/10/2026: publicação da proposta autorizada pelo mantenedor; minuta para revisão, sem vigência. A licença vigente continua sendo GPL-3.0-only. Não substituir LICENSE ou os metadados antes de resolver os pontos abaixo. Publicação do launcher continua cancelada.

## Material preparado

- LUXMC-NONCOMMERCIAL-2.0-DRAFT.md: minuta em português, com permissão não comercial, redistribuição com fonte, exploração comercial condicionada a autorização escrita, atribuição e proteção das licenças anteriores.
- A licença histórica 1.0 fica intacta no arquivo. A minuta não foi criada pela edição do texto da GPL e não é oferecida como derivação oficial de outra licença padronizada.

## Decisões propostas para revisão do titular

| Tema | Regra da minuta |
| --- | --- |
| Jogar em uso pessoal, estudar e auditar | Permitido |
| Fork gratuito para finalidade não comercial | Permitido com fonte, autoria e identificação de mudanças |
| Revenda, funcionalidades pagas ou serviço comercial por terceiros | Exige autorização separada |
| Distribuição financiada por anúncios ou pagamento obrigatório | Exige autorização separada |
| Doação espontânea sem contrapartida | Permitida |
| Serviço comercial/corpus contendo código abrangido | Exige autorização, ressalvados direitos independentes |
| Ideias, fatos e material independente sem código protegido | Não apropriados pela licença |
| Planos Plus/Pro oferecidos pelos titulares | Dependem dos direitos de cada titular e de acordos com colaboradores |
| Código recebido antes sob GPL | Concessões anteriores preservadas |

A definição de uso comercial é ampla e abrange uso interno empresarial. Confirmar se esse alcance e a exceção de doações correspondem à política desejada. A minuta trata uso pessoal para jogar como permitido; ela não define a licença do Minecraft nem concede direitos sobre conteúdo audiovisual de terceiros.

## Revisão de titularidade e compatibilidade

1. Identificar nome civil ou entidade de cada licenciante e a forma de contato para autorizações. O apelido predabr e o campo author do Git não comprovam titularidade jurídica de todos os arquivos.
2. Mapear código original, código copiado/adaptado, contribuições externas e materiais gerados com ferramentas. Registrar origem e permissões. O histórico consultado contém duas identidades de commit do mesmo apelido; isso não é uma auditoria completa de autoria ou titularidade.
3. Verificar se o Luxmc deriva de código GPL de terceiros. Se houver, a publicação da obra derivada sob restrição não comercial pode ser incompatível; obter permissão alternativa dos titulares ou separar/substituir o material de maneira juridicamente válida. Não presumir que remover cabeçalhos ou reescrever pequenas partes resolve.
4. Auditar dependências Rust, frontend, Java agent, componentes vendorizados e ativos. Não alterar seus arquivos de licença. Dependências permissivas exigem retenção de avisos; copyleft e condições especiais precisam de avaliação própria. THIRD_PARTY_NOTICES.md é um ponto de partida, não um parecer de compatibilidade desta minuta.
5. Obter concordância específica de colaboradores quando seus direitos forem necessários. A política futura de contribuições deve dizer quais direitos permitem a distribuição gratuita e a oferta comercial de Plus/Pro. Não presumir cessão ou relicenciamento por um pull request antigo.
6. Submeter a minuta e o inventário a revisão jurídica antes de usá-los como termos definitivos, especialmente pela mudança de GPL para restrição comercial. Não há conclusão de validade universal ou de possibilidade de migração do conjunto atual.

## Implementação após a revisão

Definir commit e distribuição inicial abrangidos; não reescrever tags, releases, avisos ou artefatos antigos. Se uma versão também continuar sendo oferecida sob GPL, o destinatário poderá usar os direitos da GPL, incluindo os comerciais; a minuta não elimina essa alternativa.

Preparar alteração coordenada de LICENSE, README, SECURITY, BRANDING, THIRD_PARTY_NOTICES, package.json, Cargo.toml e páginas/instaladores que exibem termos. Para metadados de licença personalizada, verificar a sintaxe suportada por cada ecossistema: Cargo aceita license-file; não inventar identificador SPDX registrado. O identificador documental provisório pode ser LicenseRef-Luxmc-Noncommercial-2.0, sem alegar reconhecimento SPDX.

Manter avisos precisos de código disponível; não anunciar open source OSI. Publicar termos de contribuição e mecanismo escrito de autorização comercial antes de aceitar novos patches sob essa política. Rever também critérios de programas de assinatura ou distribuição que exijam licença open source: a elegibilidade pode mudar.

Depois de aprovada a implementação, verificar metadados, links e avisos da distribuição e obter autorização para publicação. Esta publicação documental não cria build, instalação, deploy, release ou alteração da licença vigente. A autorização para publicar a minuta não resolve os direitos de terceiros nem equivale à ativação dos termos.

## Referências consultadas

- GNU GPLv3, seções 2, 5 e 10: https://www.gnu.org/licenses/gpl.en.html
- Texto e classificação da GPLv3: https://choosealicense.com/licenses/gpl-3.0/
- FAQ GNU, cópias sob licenças anteriores e direitos dos titulares: https://www.gnu.org/licenses/gpl-faq.en.html
- Definição de Open Source, itens 1 e 6: https://opensource.org/osd
- Exemplo de licença padronizada não comercial, consultado para comparação: https://polyformproject.org/licenses/noncommercial/1.0.0

As referências fundamentam a distinção entre os modelos; a minuta personalizada precisa de avaliação própria.
