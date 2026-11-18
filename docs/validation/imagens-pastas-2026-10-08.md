# Imagens das pastas

Cada pasta tem um botão de imagem no canto do card. Ele abre o seletor de arquivos para escolher ou substituir sua imagem. O botão de remover restaura o ícone de pasta.

PNG, JPG e WebP são convertidos em miniaturas de até 384 pixels, mantendo a proporção e a transparência. GIF é preservado no formato original. Os limites são 12 MiB para imagens estáticas e 700.000 bytes para GIF. As imagens são copiadas para o armazenamento local da interface; não dependem da localização do arquivo original.

A persistência usa `luxmc_library_folder_images_v1`, associada ao nome da pasta. Uma falha ao salvar mantém a imagem anterior e mostra uma mensagem. Arquivos inválidos são rejeitados. O seletor de imagem fica separado do botão que abre a pasta, para escolher uma foto sem entrar na pasta.

## Verificação

- `pnpm check`: zero erros e zero avisos antes e depois da queda de energia; arquivos conferidos após o retorno.
- Browser nos temas claro e escuro: upload, proporção da miniatura, persistência após reabrir, GIF preservado, rejeição de imagem inválida, preservação da imagem anterior quando o armazenamento está cheio, remoção persistente e abertura da pasta aprovados.
- Regressão de pastas: criação, arraste, conteúdo isolado, retirada da instância e preservação dos diretórios aprovados nos dois temas.

Versão mantida em 3.2.0. Sem publicação no site ou GitHub e sem substituir o aplicativo instalado.


## Instalador local

Build Windows concluído com sucesso após o retorno da energia. Instalador: `release-windows/Luxmc-3.2.0-imagens-pastas.exe`.

- Versão: 3.2.0.
- Tamanho: 20814285 bytes.
- SHA-256: `1090ff3c2fd707b2b82e72bf1f1f92d97144ac4e0b8e6f8eaa665de13ad3aa4d`.
- Cabeçalho de executável Windows validado.
- Aplicativo instalado preservado (hash antes e depois igual): True.
- Sem instalação automática ou publicação no site/GitHub.
