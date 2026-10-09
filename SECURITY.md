# Segurança do Luxmc

Baixe o launcher pelo [site oficial](https://luxmc-r92.pages.dev/download/windows) ou pelas [releases de predabr/luxmc](https://github.com/predabr/luxmc/releases). O código público permite auditoria e pode ser copiado; a licença do projeto restringe revenda e uso indevido da identidade visual.

Desde a versão 3.6, o atualizador verifica a assinatura Ed25519 de `SHA256SUMS` usando uma chave pública incluída no executável. Em seguida, confere o SHA-256 do instalador. Uma assinatura inválida, arquivo alterado ou origem não permitida impede a execução da atualização. `latest.json` também recebe uma assinatura para conferência independente. A chave privada não faz parte do repositório.

Essa assinatura identifica artefatos assinados pelo mantenedor. Não é uma assinatura Authenticode do Windows, não garante ausência de falhas e não torna uma cópia modificada impossível. Compare sempre a chave pública com a publicada no repositório oficial.

Cookies do portal usam HttpOnly, Secure e SameSite. Operações de conta limitam a origem e o volume de pedidos. Tokens Microsoft e convites privados não devem ser publicados em issues, capturas ou logs. Diagnósticos compartilhados devem remover essas informações.

Para comunicar uma vulnerabilidade, use a opção de relato privado na aba Security do repositório. Não publique credenciais ou detalhes que permitam explorar contas de usuários numa issue pública. Se o relato privado estiver indisponível, abra uma issue contendo somente um pedido de canal privado, sem os detalhes sensíveis.

Os jogos, modpacks, bibliotecas e componentes de rede mantêm seus autores e licenças. A edição Bedrock instalada pelo Luxmc passa pela instalação do Windows; o launcher não substitui nem remove a verificação de licença da Microsoft.

Alguns servidores Microsoft de pacotes antigos oferecem o conteúdo por HTTP. O catálogo e a resolução de metadados usam HTTPS; downloads tentam HTTPS primeiro e podem usar o endereço HTTP do mesmo servidor Microsoft. A identidade UWP é conferida antes da instalação e o Windows verifica a assinatura do pacote. Essa alternativa se limita aos pacotes Bedrock; o atualizador do Luxmc exige HTTPS e manifesto assinado.
