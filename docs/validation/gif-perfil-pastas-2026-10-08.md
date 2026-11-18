# Correção de GIF no perfil e nas pastas — 2026-10-08

O GIF real scuba-scuba-cat.gif em Downloads tem 6.764.271 bytes, 344×374 pixels e 150 quadros. Era rejeitado pelo limite de 700.000 bytes.

Perfil e pastas agora aceitam GIFs de entrada de até 12 MiB e reduzem os arquivos maiores em um Web Worker. O resultado permanece um GIF animado de até 700.000 bytes, compatível com o servidor e o armazenamento existente. Imagens anteriores ficam preservadas caso a conversão ou o salvamento falhe.

A cópia scuba-scuba-cat-luxmc.gif em Downloads tem 461.800 bytes, 59×64 pixels, os 150 quadros, duração de 5.010 ms e repetição infinita. O arquivo original foi preservado. A primeira cópia reduzida perdeu a extensão de repetição; o encoder e o teste foram corrigidos para exigir repetição infinita.

Teste com o arquivo real: pasta salva e persistente após reload; avatar e banner aceitos e salvos; resultado aprovado pela validação do servidor; todos os quadros, duração e loop conferidos. Testes de regressão de imagens das pastas passaram em claro e escuro.

Mudanças feitas no código local. Nenhuma publicação, alteração de versão ou substituição do aplicativo instalado. O GIF reduzido pode ser usado imediatamente no aplicativo atual.

## Integração automática no instalador

A escolha do GIF original aciona a compactação no launcher, sem exigir um arquivo previamente reduzido. Pasta, avatar e banner mostram o estado de preparação enquanto processam; o GIF só substitui a imagem ao concluir. GIFs até 12 MiB são aceitos como entrada; os maiores que 700.000 bytes são reduzidos para o limite de armazenamento/envio, com repetição contínua.

- pnpm check: zero erros e avisos.
- Teste do GIF real no servidor de desenvolvimento: aprovado.
- Teste do GIF real na interface compilada de produção: aprovado, inclusive worker, persistência de pasta e envio de avatar/banner.
- Arquivo original e aplicativo instalado preservados; versão 3.2.0 mantida.

## Entrega do instalador

Build Windows finalizado com sucesso. Arquivo: release-windows/Luxmc-3.2.0-gif-automatico.exe.

- Versão: 3.2.0.
- Tamanho: 20821180 bytes.
- SHA-256: 7d77425e0d166988b57a468f52b0eed1b463e40ff858bcf1fdb3913ab4960ac9.
- Cabeçalho PE validado.
- Aplicativo instalado preservado por conferência de hash: True.
- Nenhuma publicação no site/GitHub ou instalação automática.
