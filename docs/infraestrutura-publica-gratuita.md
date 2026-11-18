# Entrada pública do Luxmc: teste sem mensalidade

Consulta às fontes oficiais em 28/10/2026. Nenhuma conta, VM, domínio ou conexão pública foi criada nesta revisão.

O mundo e o servidor Java continuam no computador de quem hospeda. Uma máquina pública recebe o TCP do Minecraft e encaminha o tráfego por uma conexão autenticada que o launcher abre para fora. O computador do anfitrião precisa continuar ligado e conectado. O site Cloudflare Pages atual não fornece essa entrada TCP do Minecraft.

## Caminho gratuito para experimentar

1. O responsável pelo projeto cria uma conta Oracle Cloud e verifica a disponibilidade de uma VM identificada como Always Free na região escolhida. O cadastro exige um cartão aceito para verificação. Os créditos de teste de 30 dias são distintos dos recursos Always Free; não devem ser usados como base de uma promessa de gratuidade permanente.
2. A VM precisa de IP público, regras de rede adequadas e sistema Linux atualizado. O serviço de encaminhamento do Luxmc ainda precisa ser implementado e validado. Apenas criar a VM não conecta os servidores existentes.
3. Um subdomínio gratuito de um provedor como Duck DNS pode apontar para esse IP. `luxmc.duckdns.org` é somente exemplo; disponibilidade e controle não foram verificados. Um domínio registrado próprio continua tendo custo de registro e renovação.
4. O launcher recebe sua identidade de serviço, registra uma sessão autenticada de hospedagem e mantém a conexão de saída. Os convidados usam a entrada pública no Minecraft, sem conta de infraestrutura. O protocolo deve vincular cada rota ao anfitrião correto e revogar rotas encerradas.
5. Primeiro validar uma única rota e sua porta. A experiência com nomes individuais exige provisionamento DNS compatível e um gateway que resolva o destino pelo protocolo Minecraft ou por portas dedicadas. Não basta gerar nomes aleatórios na interface.
6. Testar de outra conexão de internet, incluindo CGNAT no anfitrião, reconexão, suspensão do PC, encerramento, modpacks, lista do Multijogador e limites de uso. Só então anunciar endereço público disponível no produto.

## Limites

Always Free não oferece SLA, pode estar sem capacidade na região e pode ter instâncias ociosas recuperadas pelo provedor. CPU, banda e transferência precisam ser medidos; o gateway participa do caminho de todos os jogadores e pode aumentar a latência. Não há garantia de hospedagem pública gratuita ilimitada. Uma oferta comercial Pro precisa tratar capacidade, indisponibilidade e manutenção antes de anunciar esse recurso como garantido.

DNS gratuito fornece um nome, não resolve CGNAT nem cria um túnel. O domínio do site atual também não substitui o gateway. Cloudflare Tunnel para TCP arbitrário exige cliente adicional no lado do convidado; por si só não entrega a experiência de digitar o endereço no Minecraft comum.

Uma alternativa sem VPS é disponibilizar diretamente a conexão residencial com IP público e encaminhamento de porta, quando a operadora e o roteador permitirem. Ela exige configuração por anfitrião, não atende à automação universal desejada e pode não funcionar em CGNAT.

## Fontes

- [Oracle: recursos Always Free, capacidade e recuperação por ociosidade](https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm)
- [Oracle: cadastro, cartão, teste de 30 dias e ausência de SLA](https://www.oracle.com/cloud/free/faq/)
- [Duck DNS: serviço de DNS dinâmico gratuito](https://www.duckdns.org/)
- [Cloudflare: TCP arbitrário e cliente cloudflared](https://developers.cloudflare.com/cloudflare-one/access-controls/applications/non-http/cloudflared-authentication/arbitrary-tcp/)
