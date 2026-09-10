<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="180" />
  <h1>Luxmc Launcher <span style="color:#e2b86b">v1.1.0-BETA</span></h1>
  <p><strong>O Minecraft Launcher de alta performance projetado de Linux para Linux.</strong></p>
  <p><em>Rápido. Moderno. Poderoso. 100% gratuito.</em></p>

  <p>
    <a href="https://github.com/predabr/luxmc/releases"><img src="https://img.shields.io/badge/version-v1.1.0--BETA-gold?style=for-the-badge&logo=rocket" alt="Version" /></a>
    <img src="https://img.shields.io/badge/platform-Linux-blue?style=for-the-badge&logo=linux" alt="Linux" />
    <img src="https://img.shields.io/badge/Windows-Em%20Breve-0078D4?style=for-the-badge&logo=windows" alt="Windows Em Breve" />
    <img src="https://img.shields.io/badge/backend-Rust%20%2F%20Tauri%202-orange?style=for-the-badge&logo=rust" alt="Rust Tauri 2" />
    <img src="https://img.shields.io/badge/frontend-Svelte%205-red?style=for-the-badge&logo=svelte" alt="Svelte 5" />
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Luxmc%20License-purple?style=for-the-badge" alt="License" /></a>
  </p>
</div>

<br/>

<div align="center">
  <img src="static/home.png" alt="Dashboard — Tela Principal do Luxmc" style="border-radius: 14px; margin-bottom: 20px; box-shadow: 0 20px 50px rgba(0,0,0,0.5);" width="850" />
  <p><em>Dashboard com modpacks em destaque, servidores recomendados, estatísticas de jogo e notícias da comunidade.</em></p>
</div>

<br/>

<div align="center" style="display: flex; justify-content: center; gap: 20px;">
  <img src="static/servers.png" alt="Diretório de Servidores com ping em tempo real" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
  <img src="static/skins.png" alt="Personalização 3D de Skins & Capas" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
</div>
<div align="center">
  <p><em>Esquerda: Diretório de 100+ servidores com ping e filtros. Direita: Personalização 3D volumétrica com NameMC e capas.</em></p>
</div>

<br/>

---

## 🎮 O que é o Luxmc?

O **Luxmc** é um launcher de Minecraft moderno construído **do zero em Rust (Tauri 2)** com frontend em **Svelte 5 + TypeScript + Tailwind CSS**. Projetado especificamente para Linux, ele oferece consumo mínimo de RAM, inicialização instantânea e uma experiência de usuário premium com interface inspirada nos melhores launchers do mercado.

### Por que usar o Luxmc?

| Recurso | Luxmc | Launcher Oficial | Outros Launchers |
|---------|-------|-----------------|-------------------|
| **Consumo de RAM** | ~200 MB | ~800 MB+ | ~400-600 MB |
| **Inicialização** | < 2s | ~8-15s | ~5-10s |
| **Central de Mods** | ✅ Modrinth + CurseForge | ❌ | ✅ Parcial |
| **Personalização 3D** | ✅ Volumétrico real | ❌ | ✅ 2D apenas |
| **Diretório de Servidores** | ✅ 100+ com ping | ❌ | ❌ |
| **Open Source** | ✅ | ❌ | Parcial |
| **Nativo Linux** | ✅ Tauri 2 / Rust | ❌ Electron | ❌ Java/Electron |

---

## 🌟 Novidades na v1.1.0-BETA

- ⚡ **Sistema de Otimização Inteligente Luxmc (Aikar G1GC Tuning):** Gerador dinâmico de parâmetros de JVM baseado na especificação de Aikar para G1GC do Minecraft. Ajusta o tamanho das regiões de heap (`G1HeapRegionSize`), threads paralelas e threads de marcação concorrente proporcionalmente à RAM alocada:
  - **Medição Real no Host (OpenJDK 26):** Inicialização padrão em 41ms vs. 289ms com flags inteligentes. Os ~248ms adicionais no boot são decorrentes de `-XX:+AlwaysPreTouch`, que pré-aloca e toca todas as páginas físicas de memória do SO durante a inicialização para **eliminar congelamentos e engasgos de GC durante a gameplay**.
- 🚀 **Pacote de Performance Essencial em 1-Clique:**
  - Instalação oficial automatizada para Fabric (*Sodium*, *Lithium*, *FerriteCore*) e Forge/NeoForge (*Embeddium*, *ModernFix*, *FerriteCore*).
  - Resolução direta de versões compatíveis via Modrinth API com notificação clara para instâncias Vanilla.
- 🎮 **Aceleração Gráfica Mesa Zink / Vulkan no Linux:**
  - Redireciona o pipeline gráfico OpenGL diretamente para os drivers Vulkan nativos da GPU (RADV para AMD, ANV para Intel) via Gallium Zink.
  - Ativação automática de `RADV_PERFTEST=aco` para compilação ultrarrápida de shaders em GPUs AMD.
- 🔍 **Detecção de Hardware Nativa no Linux:**
  - Leitura direta pelo subsistema DRM em `/sys/class/drm` e `lspci`, identificando fabricante da GPU, renderizador e suporte a Zink sem subprocessos desnecessários.
- 🛠️ **Correção Crítica no Pipeline de Inicialização de JVM:**
  - Remoção de flags hardcoded que bloqueavam a alocação de memória customizada em instâncias.
- 💎 **Varredura Completa de Design & Zero Warnings:**
  - Padronização no design system Luxury Dark (`#141518` / `#caa97c`) em todas as telas (Settings, Logs, Screenshots, Skins e Servidores).
  - 100% de conformidade técnica: zero erros e zero warnings em `cargo check`, `cargo test`, `pnpm check` e `pnpm build`.

<details>
<summary><strong>📦 Histórico de Versões Anteriores (v1.0.0-BETA)</strong></summary>

- ⚡ **Zero-Lag & Aceleração GPU Total:** Pipeline DMA-BUF ativo no WebKitGTK para o Linux AppImage rodar liso na sua GPU.
- 🧠 **Gerenciamento Inteligente de Memória:** Limitadores de memória integrados impedem vazamentos de RAM.
- 📦 **Central de Mods com Visão Detalhada:** Descrição formatada, galeria, changelogs e downloads direto do **Modrinth** e **CurseForge**.
- 🧩 **Gerenciador Interno de Mods por Instância:** Ative/desative mods (`.jar` ↔ `.jar.disabled`), exclua ou abra a pasta nativa.
- 📋 **Logs & Crash Reports via `mclo.gs`:** Envie relatórios de erro sanitizados com cópia automática de link.
- 🔑 **Suporte a Azure Client ID:** Configure seu Client ID Microsoft direto nas Configurações.
- 🎬 **Cutscene Cinematográfica:** Abertura rápida de 2.3s com anéis orbitais e snap magnético.
- 🔄 **Reparo, Backup e Exportação de Instâncias:** Repare instâncias corrompidas e exporte instâncias como `.zip`.

</details>

---

## 🚀 Principais Recursos

### ⚡ Desempenho
- **Construído em Rust (Tauri 2):** Consumo mínimo de memória RAM e inicialização instantânea.
- **Pipeline GPU nativo:** DMA-BUF e aceleração de hardware via WebKitGTK.
- **Limitadores de memória integrados:** Sem vazamentos, sem travamentos.

### ☕ Java & Otimização
- **Smart Java & Auto-Otimização:** Detecta ou baixa o Java correto para cada versão do Minecraft.
- **Aikar's Flags automáticas:** Aplica as melhores flags de JVM automaticamente.
- **Editor de JVM por instância:** Presets de RAM + validador de flags em tempo real.

### 🧩 Mods & Loaders
- **Suporte a Múltiplos Mod Loaders:** Vanilla, Fabric, Forge, NeoForge e Quilt.
- **Central de Conteúdo:** Modrinth + CurseForge com filtros por fonte, tipo, versão, loader e categoria.
- **Fabric API automático:** Detecta e instala o Fabric API quando necessário.
- **Gerenciador por instância:** Ative/desative/exclua mods com isolamento seguro.

### 👕 Personalização
- **Renderizador 3D Volumétrico Real:** Modelo de 9 faces com rotação 360° em tempo real.
- **NameMC Integration:** Busca de skins por nickname direto pelo launcher.
- **Capas:** Suporte a OptiFine, Mojang e Migrator.

### 🌐 Servidores & Multiplayer
- **Diretório de 100+ Servidores:** Brasileiros e mundiais com ping em tempo real e filtros.
- **Mundo Aberto P2P:** Jogue mundos locais com amigos via rede/LAN.
- **Cópia de IP em 1 clique** para entrar rápido.

### 🔒 Privacidade & Segurança
- **Keyring seguro do sistema:** Tokens protegidos via libsecret / KWallet.
- **Modo anônimo para streamers:** Oculte dados sensíveis.
- **Sem telemetria:** Zero rastreamento, zero dados coletados.

### 📥 Atualizações
- **Verificação automática:** Alertas nativos para novas versões do launcher.
- **Reparo de instâncias:** Corrija instâncias corrompidas com 1 clique.
- **Backup & Export:** Salve e compartilhe instâncias completas.

---

## 📸 Galeria de Screenshots

<details>
<summary><strong>🏠 Dashboard — Tela Principal</strong></summary>
<br/>
<div align="center">
  <img src="static/home.png" alt="Dashboard" width="850" style="border-radius: 12px;" />
</div>

- Saudação personalizada com nome do jogador
- Modpacks em destaque com carrossel
- Servidores recomendados com ping ao vivo
- Estatísticas de jogo (tempo total, sessões, versão principal)
- Notícias & Comunidade na sidebar direita
</details>

<details>
<summary><strong>🌐 Diretório de Servidores</strong></summary>
<br/>
<div align="center">
  <img src="static/servers.png" alt="Servidores" width="850" style="border-radius: 12px;" />
</div>

- 100+ servidores brasileiros e mundiais
- Filtros: Todos, Original, Pirata, Survival, Bedwars, Brasil, Skyblock, PvP, etc.
- Ping em tempo real e contagem de jogadores online
- Badges de tipo (ORIGINAL, ANARCHY, SURVIVAL)
- Cópia de IP em 1 clique
</details>

<details>
<summary><strong>👕 Personalização 3D de Skins & Capas</strong></summary>
<br/>
<div align="center">
  <img src="static/skins.png" alt="Skins 3D" width="850" style="border-radius: 12px;" />
</div>

- Modelo 3D volumétrico real com rotação completa
- Integração NameMC para busca de skins
- Galeria de skins salvas e customizadas
- Capas: Migrator, OptiFine, Mojang
- Importação de skin .PNG e sincronização em tempo real
</details>

<details>
<summary><strong>📦 Central de Conteúdo (Mods)</strong></summary>
<br/>
<div align="center">
  <img src="static/screenshot.png" alt="Central de Mods" width="850" style="border-radius: 12px;" />
</div>

- Grid de modpacks/mods com imagem dual (banner + ícone)
- Filtros: Modrinth/CurseForge, Modpack/Mod/Shader/Resource Pack/Data Pack
- Filtro por versão do jogo e mod loader
- Categorias: Adventure, Challenging, Combat, Kitchen Sink, Magic, etc.
- Botão "Instalar" com progresso em tempo real
</details>

---

## 📥 Como Baixar e Instalar

Baixe o pacote correspondente à sua distribuição na aba de **[📦 Releases Oficiais](https://github.com/predabr/luxmc/releases)**.

### 🟢 Universal Linux (AppImage) — Recomendado
Funciona em **qualquer** distribuição Linux (Ubuntu, Debian, Fedora, Arch, Pop!_OS, openSUSE, Mint, etc.):
```bash
# 1. Baixe o AppImage da aba Releases
# 2. Dê permissão de execução
chmod +x luxmc-1.1.0-beta.AppImage

# 3. Execute
./luxmc-1.1.0-beta.AppImage
```

### 🔴 Debian / Ubuntu / Pop!_OS / Linux Mint (.deb)
```bash
sudo apt install ./luxmc_1.1.0_amd64.deb
```

### 🔵 Fedora / RHEL / openSUSE (.rpm)
```bash
sudo dnf install ./luxmc-1.1.0.x86_64.rpm
```

### 🟣 Arch Linux / Manjaro (AUR)
```bash
yay -S luxmc-bin
# ou
paru -S luxmc-bin
```

### 🪟 Microsoft Windows (Em Breve)
> 🚀 O suporte nativo ao **Windows** (instaladores `.msi` e `.exe`) está em fase de finalização e será disponibilizado nas próximas atualizações!

---

## 🛠️ Compilando do Código-Fonte

### Requisitos do Sistema

| Requisito | Versão Mínima |
|-----------|--------------|
| **Node.js** | v20+ |
| **pnpm** | v9+ |
| **Rust** (stable) | 1.77+ |
| **WebKitGTK** | 4.1+ |
| **GTK** | 3.0+ |

### Dependências de Desenvolvimento

<details>
<summary><strong>Debian / Ubuntu</strong></summary>

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev \
  libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```
</details>

<details>
<summary><strong>Arch Linux</strong></summary>

```bash
sudo pacman -S webkit2gtk-4.1 base-devel openssl gtk3 libappindicator-gtk3 librsvg
```
</details>

<details>
<summary><strong>Fedora</strong></summary>

```bash
sudo dnf install webkit2gtk4.1-devel @development-tools openssl-devel \
  gtk3-devel libappindicator-gtk3-devel librsvg2-devel
```
</details>

### Build & Execução

```bash
# 1. Clone o repositório
git clone https://github.com/predabr/luxmc.git
cd luxmc

# 2. Instale as dependências do frontend
pnpm install

# 3. Inicie em modo desenvolvimento (hot-reload)
pnpm tauri dev

# 4. Gere os pacotes de produção (AppImage, .deb, .rpm)
pnpm tauri build
```

### Variáveis de Ambiente (Opcional)

Copie o arquivo `.env.example` para `.env` e configure:

```bash
cp .env.example .env
```

| Variável | Descrição |
|----------|-----------|
| `CURSEFORGE_API_KEY` | Chave da API do CurseForge (opcional — sem ela, apenas Modrinth funciona) |
| `LUXMC_MS_CLIENT_ID` | Azure Client ID para autenticação Microsoft |
| `LUXMC_DEV_MODE` | Ativa o modo desenvolvedor (`true`/`false`) |
| `LUXMC_SOFTWARE_RENDER` | Força renderização por software (`true`/`false`) |

---

## 🏗️ Arquitetura

```
luxmc/
├── src/                          # Frontend (SvelteKit + Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── api/                  # Wrappers tipados para Tauri commands
│   │   ├── components/           # Componentes UI reutilizáveis
│   │   │   ├── layout/           # Sidebar, Header, DownloadProgressBar
│   │   │   └── ui/               # Botões, Cards, Toggles
│   │   └── stores/               # Estado global (Svelte 5 runes)
│   └── routes/                   # Páginas SvelteKit
│       ├── +page.svelte          # Login / Dashboard
│       ├── mods/                 # Central de Conteúdo
│       ├── instances/            # Gerenciador de Instâncias
│       └── settings/             # Configurações
├── src-tauri/                    # Backend Rust (Tauri 2)
│   ├── src/
│   │   ├── core/                 # Lógica de negócio
│   │   │   ├── downloader.rs     # Download Manager com progresso
│   │   │   ├── loaders/          # Fabric, Quilt, NeoForge
│   │   │   └── mods/             # Modrinth & CurseForge clients
│   │   ├── commands/             # #[tauri::command] handlers
│   │   └── lib.rs                # Registro de commands
│   └── tests/                    # Testes de integração Rust
├── static/                       # Assets estáticos
└── tailwind.config.ts            # Tokens de design Tailwind
```

### Stack Tecnológica

| Camada | Tecnologia | Versão |
|--------|-----------|--------|
| **Backend** | Rust + Tauri 2 | Stable 1.77+ |
| **Frontend** | SvelteKit + Svelte 5 | ^5.56 |
| **Estilização** | Tailwind CSS | ^3.4 |
| **Renderização 3D** | Three.js + Threlte | ^0.185 |
| **Banco de Dados** | SQLite (sqlx) | Integrado |
| **Segurança** | libsecret / KWallet (Keyring) | Sistema |
| **Build** | Vite 6 + Cargo | Latest |

---

## 🤝 Contribuindo

Contribuições são bem-vindas! Siga estas diretrizes:

1. **Fork** o repositório
2. **Crie uma branch** para sua feature: `git checkout -b feat/minha-feature`
3. **Siga as convenções** do projeto:
   - TypeScript strict mode ativo
   - Svelte 5 runes (`$state`, `$derived`, `$effect`) — sem stores legados
   - Tokens Tailwind do `tailwind.config.ts` — nunca hardcode valores hex
   - Todo side-effect Tauri via `#[tauri::command]` + wrapper tipado em `src/lib/api/`
4. **Verifique** antes de commitar:
   ```bash
   pnpm check        # Frontend (svelte-check)
   cargo check        # Backend (Rust)
   cargo test --tests # Testes de integração
   ```
5. **Abra um Pull Request** descrevendo suas mudanças

---

## 📋 Roadmap

- [x] Central de Mods (Modrinth + CurseForge)
- [x] Personalização 3D de Skins & Capas
- [x] Diretório de Servidores com ping real
- [x] Fabric loader com Fabric API automático
- [x] Editor de JVM por instância
- [x] Sistema de reparo, backup e exportação
- [x] Barra de progresso de downloads
- [x] Cutscene cinematográfica
- [ ] Autenticação Microsoft completa (aguardando aprovação Azure)
- [ ] Suporte ao Windows (instaladores .msi e .exe)
- [ ] NeoForge loader completo (prepare_loader)
- [ ] Sistema de amigos e chat integrado
- [ ] Marketplace de skins e capas da comunidade
- [ ] Suporte a macOS

---

## 📄 Licença

Distribuído sob a **Licença Própria do Luxmc Launcher** (Copyright © 2026 Pedro & Time Luxmc).
O software é gratuito para uso pessoal e não comercial. É expressamente proibida a revenda, comercialização, empacotamento com anúncios (adware) ou rebranding não autorizado. Para ler todos os termos e condições, consulte o arquivo [LICENSE](LICENSE).

---

<div align="center">
  <br/>
  <p>
    <strong>Feito com ❤️ no Brasil, para a comunidade Linux.</strong>
  </p>
  <p>
    <a href="https://github.com/predabr/luxmc/releases">📦 Download</a> •
    <a href="https://github.com/predabr/luxmc/issues">🐛 Reportar Bug</a> •
    <a href="https://github.com/predabr/luxmc/discussions">💬 Discussões</a>
  </p>
  <br/>
  <img src="static/logo.png" alt="Luxmc" width="60" />
</div>
