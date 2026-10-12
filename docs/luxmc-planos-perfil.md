# Luxmc Plus e Pro — personalização

Os benefícios adicionais focam em perfil e organização. Jogar, usar modpacks, segurança e a rede com amigos continuam disponíveis no Free. Servidores hospedados ficam fora do escopo atual.

| Recurso | Free | Plus | Pro |
| --- | --- | --- | --- |
| Nome, status, bio, avatar, capa e GIF | Incluído | Incluído | Incluído |
| Paletas para o perfil | Luxmc | 6 | 6 |
| Molduras | Original | Original, Halo e Contorno duplo | Todas, incluindo Órbita |
| Efeitos visuais | Sem efeito | Sem efeito | Brilho da capa e Pulso do avatar |
| Composição | Clássica | Clássica | Clássica e centralizada |
| Favoritos no perfil | 8 | 16 | 24 |
| Coleções públicas | 6 | 12 | 24 |
| Perfis de aparência sincronizados | 0 | 6 | 20 |
| Selo do plano | — | Plus | Pro |

Os preços cadastrados permanecem R$ 9,90 e R$ 19,90 por 30 dias após aprovação. Os novos pedidos seguem pausados. A implementação não concede nem renova assinaturas por conta própria.

A assinatura visual usa o mesmo registro de perfil no site e no launcher. As escolhas podem ser alteradas na página de planos ou no editor do launcher. O servidor consulta o plano da identidade autenticada a cada gravação e leitura. Molduras chegam também à lista de amigos, à barra lateral e à sala.

Se o plano vencer, as opções pagas deixam de ser exibidas, mas permanecem guardadas para renovação. Favoritos e coleções existentes ficam preservados; criar novos itens segue o limite ativo. Clientes anteriores recebem até oito favoritos e preservam os itens excedentes e o estilo ao salvar as informações que já conhecem. O cliente atualizado solicita o limite de leitura ampliado.

Tags administrativas, incluindo dono e testador, continuam sendo atribuídas somente pelo dono. Assinar um plano não dá acesso ao painel administrativo nem permite editar essas tags.

A nova coluna de banco está em `website/migrations/0016_profile_style.sql`. A publicação precisa aplicar essa migração antes de disponibilizar o código de API que usa os campos novos. O launcher também precisa de um novo executável para transportar a assinatura visual na gravação nativa.

As alterações estão locais, sem mudança de versão ou publicação. Os resultados de validação estão em `docs/validation/plan-benefits`. A compilação Rust permanece bloqueada nas dependências pelo Windows; os testes de navegador usam o frontend real com respostas nativas simuladas.
