# Editor de perfil Luxmc · 11/10/2026

O editor foi separado em Informações, Aparência e Vitrine. O painel oferece prévia ao vivo, prévia completa, contadores dos campos, sugestões de status e biografia, quatro capas Luxmc, enquadramento de imagens com zoom e posição, escolha de favoritos da biblioteca, ordenação de favoritos e coleções de modpacks.

O salvamento usa o contrato existente do perfil público. GIFs mantêm o caminho de processamento animado; imagens estáticas são enquadradas e exportadas em WebP. Fotos acima de 4096 pixels são reduzidas antes do enquadramento e seus bitmaps são liberados quando a edição termina. As tags permanecem controladas pelo dono e o plano continua vindo do servidor.

O editor avisa ao fechar um rascunho, mantém as alterações quando o salvamento falha, preserva a seção ao voltar da prévia e oferece Ctrl + S. Reabrir uma edição começa em Informações.

## Verificação

- `pnpm check`: 0 erros e 0 avisos.
- `pnpm build`: compilação de produção concluída.
- `tests/profile-editor.browser.cjs`: aprovado nos temas claro e escuro, incluindo foto de 8000 pixels, aplicação real do enquadramento, formato aceito pelo validador do servidor, GIF preservado, ordem dos favoritos, coleção, falha de conexão, recuperação do rascunho, confirmação ao fechar, reabertura e janela de 720 pixels sem transbordamento.
- Testes de traduções: 3 aprovados em português, inglês e espanhol.
- As prévias usam dados simulados e não alteraram perfis reais.

As mudanças estão no projeto local. A instalação e a publicação de uma nova versão não fazem parte desta alteração.
