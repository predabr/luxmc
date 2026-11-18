# Rede virtual, Bedrock e armazenamento — 8 de outubro de 2026

Este documento registra a revisão 3, que falhou no teste do usuário por ausência de Packet.dll. Foi substituída pela revisão 4 documentada em lan-revision4-2026-10-08.md; as afirmações de preparação da LAN abaixo não descrevem a revisão corrigida.

Revisão local de teste: Minecraft launcher 3.5.0, revisão 3. Publicação no GitHub e no site permanece suspensa a pedido do usuário.

## Rede virtual

A tela de hospedagem passa a oferecer criar e entrar numa LAN virtual pelo próprio Luxmc. O motor externo é EasyTier v2.6.4, com adaptação Wintun no Windows x64. O pacote e cada um dos três componentes permitidos são conferidos por SHA-256 fixado na implementação.

O instalador de componentes lê o ZIP em memória e extrai somente easytier-core.exe, easytier-cli.exe e wintun.dll. WinDivert64.sys, Packet.dll e os executáveis web não são extraídos. Uma pasta de runtime que contenha WinDivert64.sys ou Packet.dll é recusada. A proteção do computador continua ativa.

O hash da DLL Wintun selecionada foi também comparado à DLL AMD64 do SDK oficial Wintun 0.14.1 e coincide. Nessa comparação, nenhum binário ou driver foi extraído para o disco. As licenças do EasyTier, GPL e Wintun acompanham os componentes instalados.

O adaptador é criado somente pela ação de conexão e requer autorização do Windows. Um processo auxiliar do próprio launcher executa o motor verificado, acompanha a identidade e o tempo de criação do processo do launcher e encerra a rede ao sair, ao fechar o launcher ou ao expirar a sessão. A senha fica fora dos argumentos do processo. As interfaces RPC ficam restritas ao loopback.

Os convites duram 24 horas. O painel apresenta o IP virtual e os pares encontrados. O jogo usa Conexão direta com IP virtual e a porta informada pelo Minecraft ao abrir para LAN. A sessão nativa permanece ativa ao trocar de aba. Os convites antigos informam que é necessário criar um convite novo.

A rede usa o nó público comunitário DreamLife, publicado pelo operador em https://easytier.dreamlife.indevs.in/, para descoberta e eventual retransmissão. O acesso TCP a dreamlife.indevs.in:11010 foi confirmado neste computador. Os antigos nomes public.easytier.cn e public.easytier.top não resolveram nesta rede e foram descartados. Disponibilidade e latência desse serviço comunitário são dependências externas; o acesso TCP não comprova toda a negociação VPN. O encaminhamento para redes estrangeiras é desativado no cliente.

Ainda é necessário testar o adaptador real e uma partida entre computadores em redes diferentes. Os testes de interface usam respostas controladas, sem carregar drivers, e não comprovam essa conexão.

## Alerta do antivírus

Na investigação inicial, o ZIP completo foi extraído em Temp. O antivírus marcou WinDivert64.sys como aplicação potencialmente indesejada. A origem e o SHA-256 do ZIP correspondiam à distribuição oficial; isso não é motivo para restaurar a detecção ou classificá-la automaticamente como falso positivo.

A inspeção não encontrou processos EasyTier ativos nem drivers WinDivert/Wintun registrados. A tentativa de apagar os artefatos temporários, inclusive após autorização explícita do usuário, foi rejeitada pela revisão automática de segurança. A remoção não pode ser declarada concluída. O usuário deve manter a detecção em quarentena e usar a exclusão no antivírus.

## Bedrock

Antes da criação de instância, o usuário escolhe Java ou Bedrock. O BedrockLauncher é conectado escolhendo seu executável e sua pasta de dados; o Luxmc lê apenas metadados das instalações existentes. Conta e licença continuam sendo responsabilidade do provedor e do jogo.

Cada vínculo conserva seu perfil e instalação exatos. Diretórios absolutos, caminhos que escapem do provedor e instalações com pasta compartilhada são rejeitados. Remover um vínculo preserva os mundos e os arquivos do provedor. As bibliotecas da página inicial e de instâncias exibem esses vínculos separadamente.

A abertura específica usa o protocolo de argumentos publicado pelo BedrockLauncher. O código do provedor examinado apresenta uma limitação na leitura inicial desses argumentos. Se ele encerrar antes de confirmar a abertura, o Luxmc abre o provedor sem argumentos e informa que o usuário deve escolher a instalação e clicar em Jogar. O lançamento real de uma instalação Bedrock licenciada, o catálogo histórico disponível e a compatibilidade GDK ainda precisam ser verificados no provedor instalado; não há garantia de todas as versões.

## Armazenamento e site

O total passa a deduplicar hard links pelo identificador do arquivo. O tamanho por instância continua mostrando seus próprios arquivos, enquanto o total contabiliza cada arquivo compartilhado uma vez. O cache inclui o diretório de cache real do launcher. Pastas Bedrock vinculadas entram na medição sem virar perfis Java. As medidas usam bytes lógicos dos arquivos, não a alocação física NTFS nem todo o consumo do disco.

O site conserva o estilo existente, acrescentando entrada suave dos elementos e transição das imagens do tour. A seleção espera a imagem carregar e descarta respostas antigas. Pausa, aba oculta e preferência de movimento reduzido interrompem a animação. Os arquivos gerados estão preparados localmente; não foram publicados nesta revisão.

## Validação

- Cargo check passou para a implementação nativa; a compilação mantém cinco avisos existentes.
- Suíte nativa final: 163 aprovados, dois ignorados. Inclui extração que ignora WinDivert e caminhos fora da lista, rejeição de conteúdo alterado, medição Bedrock e deduplicação de hard links.
- Testes de interface: LAN, convite, navegação, saída, armazenamento, capas e criação Java/Bedrock aprovados.
- Parsing dos links: 21 testes aprovados.
- Website: 45 testes aprovados; compilação do cliente e servidor aprovada. Após isso, a suíte de interface passou em seis testes, incluindo a corrida entre duas imagens do tour.
- Svelte check final: zero erros e zero avisos. Cargo check final passou.
- Identificação de revisões do atualizador: 16 testes aprovados. A versão continua 3.5.0, com revisão local 3.
- Instalador NSIS local em compilação com uma tarefa Cargo e prioridade abaixo do normal.

Resultados finais e o hash do instalador serão registrados após a conclusão da compilação.
