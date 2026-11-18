# Plano e execução: pastas e capas

## Plano

1. Retirar as pastas padrão da tela inicial e a associação automática de novas instâncias por loader.
2. Permitir criar pastas com nome próprio, mover instâncias por arraste ou seletor e voltar para “Sem pasta”. Preservar os diretórios dos jogos.
3. Separar capa e logo no catálogo: capa larga, logo inteira em espaço próprio; usar fundo suave quando o projeto só oferece uma logo.
4. Ampliar a logo nos cards da biblioteca e permitir duas linhas para nomes longos.
5. Validar persistência, arraste, proporções, espaçamento e contraste nos dois temas.

## Implementado

As pastas Vanilla e Modded deixam de aparecer por associação automática. Pastas já criadas pelo usuário continuam disponíveis, inclusive quando ele escolhe esses nomes. Os registros antigos não são apagados. Novas instâncias ficam sem associação automática; mover para uma pasta altera a organização, sem mover arquivos do jogo.

O registro antigo de grupos é migrado para `luxmc_library_folders_v2`, retirando os nomes padrão que a versão anterior inseria automaticamente na lista salva. Pastas personalizadas são preservadas. A migração foi verificada com uma lista antiga contendo Vanilla e Modded.

O catálogo deixa de centralizar uma imagem quadrada pequena na área da capa. A capa preenche a área larga; a logo permanece inteira e separada. Quando só existe uma logo, ela fornece o fundo desfocado e é apresentada inteira no espaço próprio. A biblioteca apresenta logos maiores e nomes em até duas linhas.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `tests/catalog-profile-groups.browser.cjs`: aprovado em claro e escuro, incluindo criação manual, ausência das pastas padrão, arraste, reabertura e preservação dos diretórios.
- `tests/folders-artwork.browser.cjs`: aprovado em claro e escuro, incluindo logos inteiras, ausência de sobreposição com títulos e botões, scroll e geometria do instalador.
- Capturas revisadas visualmente nos dois temas.

O teste amplo `final-polish.browser.cjs` passou pelas verificações visuais, mas parou numa espera de notícia fora deste escopo. O teste dedicado termina após a verificação do instalador e passou nos dois temas.

Versão mantida em 3.2.0. Correção local: sem publicação no GitHub ou site e sem alterar o aplicativo instalado.

## Instalador local

Compilação Windows de produção concluída com sucesso, incluindo o verificador de integridade. Arquivo: `release-windows/Luxmc-3.2.0-pastas-e-capas.exe`, com 20.810.017 bytes. O SHA-256 está no arquivo `.exe.sha256` ao lado do instalador. O frontend compilado contém a migração `luxmc_library_folders_v2`.

O executável instalado permaneceu com SHA-256 `680e6ac799828d8939bb10bcde0607950cc59f2b3f31ac585c23318f2f924ea6`, idêntico ao registrado antes da compilação. O instalador foi gerado e conferido; não foi executado neste PC.
