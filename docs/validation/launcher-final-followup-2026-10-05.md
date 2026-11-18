# Lux MC Launcher — revisão final e nova gravação

Os filtros da tela inicial agora usam uma única superfície, borda e seta. A biblioteca de instâncias apresenta tempo jogado, mods e RAM em três colunas com ícones e valores legíveis.

Explorar NameMC abre diretamente o catálogo nativo. A importação por link continua disponível como ação secundária e alternativa em caso de falha. A seleção salva a textura na coleção sem aplicar a conta automaticamente. Cada skin salva guarda sua própria seleção de capa; trocar a skin e reiniciar o launcher preserva essa associação. Aplicar skin e capa permanece disponível mesmo sem alterações.

Wallpapers importados são copiados para a pasta de dados locais do Luxmc em `wallpapers`, fora do cache. O original permanece intacto. O nome legível é preservado, arquivos idênticos são reaproveitados e referências anteriores são migradas quando o original ainda existe. As prévias usam proporção de vídeo e exibem um frame pelo player quando o FFmpeg não está disponível.

## Verificação

- `pnpm check`: zero erros e zero avisos.
- `cargo check`: aprovado; quatro avisos preexistentes.
- Teste nativo de importação: cópia, deduplicação, exclusão do original e leitura do arquivo preservado.
- Browser: filtros, catálogo direto, importação, capa por skin, troca e recarga, wallpaper gerenciado e vídeo sem FFmpeg; zero erros de página.
- Regressões de reaplicação de aparência, sincronização Microsoft, Friends responsivo, hospedagem, armazenamento e recuperação de carregamento sem resposta.
- P2P nativo: sete testes aprovados; teste adicional pelo relay público aprovado. Inclui transferência criptografada, participantes, bloqueio, renovação de convite e encerramento.

O teste de relay usa dois endpoints neste Windows. A conexão Minecraft entre dois computadores independentes ainda depende do teste com o amigo e de versões/mods compatíveis.

## Entrega

O instalador mantém o nome `Lux MC Launcher.exe`. A entrega inclui instalação silenciosa, preservação do banco/contas/configurações e verificação dos comandos no binário instalado.

Após finalizar e instalar o launcher, as 25 cenas foram gravadas novamente. O catálogo público NameMC foi capturado da janela nativa instalada e recortado para excluir anúncios. Os vídeos mostram a interface real com dados isolados de demonstração, identificados na tela. Incluem narração em português, legendas incorporadas e SRT, trilha original e animações de edição. Não apresentam a sala demonstrativa como um teste real entre dois jogadores.

O Shorts final tem 55 segundos, em 1080 × 1920; o tour completo tem 4 minutos e 27 segundos, em 1920 × 1080. Ambos usam H.264 a 30 fps e AAC estéreo. A decodificação integral passou e as folhas de contato e frames finais de biblioteca e NameMC foram inspecionados. O Shorts anterior em Downloads foi substituído; o tour e os dois SRT foram entregues com os mesmos nomes previstos. Os hashes das cópias foram conferidos e registrados em `final-video-delivery.json`.

Os resultados finais de mídia ficam em `docs/media/2026-10-05/exports/{shorts,normal}-verification.json`; os hashes do instalador e a verificação nativa ficam em `redesign-delivery.json` e `redesign-native-verify.json`.
