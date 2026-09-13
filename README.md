<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="180" />
  <h1>Luxmc Launcher <span style="color:#e2b86b">v1.5.5-BETA</span></h1>
  <p><strong>O Minecraft Launcher moderno, inteligente e de alta performance.</strong></p>
  <p><em>Rápido. Inteligente. Estável. 100% gratuito e open-source.</em></p>

  <p>
    <a href="https://github.com/predabr/luxmc/releases"><img src="https://img.shields.io/badge/version-v1.5.5--BETA-gold?style=for-the-badge&logo=rocket" alt="Version" /></a>
    <img src="https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20Arch-blue?style=for-the-badge&logo=linux" alt="Multiplatform" />
    <img src="https://img.shields.io/badge/Windows-Nativo%20EXE-0078D4?style=for-the-badge&logo=windows" alt="Windows EXE" />
    <img src="https://img.shields.io/badge/Arch%20Linux-AUR%20PKGBUILD-1793D1?style=for-the-badge&logo=archlinux" alt="Arch Linux" />
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
  <img src="static/skins.png" alt="Personalização 3D de Skins & Capas" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
  <img src="static/home.png" alt="Organizador de Layout e Instâncias" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
</div>
<div align="center">
  <p><em>Esquerda: Personalização 3D volumétrica com catálogo HD, NameMC e injeção nativa no jogo. Direita: Dashboard fluido com transições e organizador modular.</em></p>
</div>

<br/>

---

## 🎮 O que é o Luxmc?

O **Luxmc** é um launcher de Minecraft moderno construído **do zero em Rust (Tauri 2)** com frontend em **Svelte 5 + TypeScript + Tailwind CSS**. Projetado com foco primário em estabilidade no Linux e suporte nativo ao Windows, ele oferece consumo reduzido de RAM, inicialização ultra-rápida, launcher inteligente com auto-dimensionamento de memória e uma experiência visual premium.

### Por que usar o Luxmc?

| Recurso | Luxmc | Launcher Oficial | Outros Launchers |
|---|---|---|---|
| **Consumo de RAM** | ~180-220 MB | ~800 MB+ | ~400-600 MB |
| **Inicialização** | < 2s | ~8-15s | ~5-10s |
| **Central de Mods** | ✅ Modrinth + CurseForge (4-level fallback) | ❌ | ✅ Parcial |
| **Launcher Inteligente** | ✅ Auto-alocação de RAM & Flags G1GC | ❌ | ❌ |
| **Personalização 3D** | ✅ Volumétrico real + 13 Capas + Injeção Nativa | ❌ | ✅ 2D apenas |
| **Organizador de Layout** | ✅ Drag-and-drop customizável | ❌ | ❌ |
| **Cancelamento de Modpacks** | ✅ Suporte com 1-clique | ❌ | ❌ |
| **Chat & P2P Multiplayer** | ✅ Socket TCP Direto | ❌ | ❌ |
| **Nativo Linux & Windows** | ✅ Tauri 2 / Rust | ❌ Electron | ❌ Java/Electron |

---

## 🌟 Novidades na v1.5.6-BETA

- 🚀 **Resolução Definitiva de Inicialização no CurseForge:**
  - Tratamento inteligente da versão base (`clean_req_ver`) no manifesto da Mojang, impedindo que nomes compostos (ex: `1.20.1-forge-47.3.0`) causem o erro "Version not found" que fechava o jogo instantaneamente.
  - Varredura e extração robusta de `version.json` diretamente do instalador do Forge na memória.
  - Deduplicação do classpath entre o cliente vanilla e o cliente embutido do NeoForge.

- 👕 **Injeção Perfeita de Skins e Capas em Runtime:**
  - Correção definitiva do canal alfa nas costas do jogador (tronco, braços, pernas e cabeça), eliminando bugs visuais e texturas invisíveis no jogo.
  - Ativação automática e forçada de todas as camadas do modelo (`modelPart_jacket`, `modelPart_cape`, `modelPart_hat`, mangas e calças) no `options.txt`.
  - Injeção da textura de capa para OptiFine e loaders modernos via resourcepack dinâmico em tempo de execução.

- 🛡️ **Persistência Completa de Skins e Capas Locais:**
  - Armazenamento permanente dos arquivos `.png` em disco (`~/.local/share/luxmc/skins/` e `capes/`) e registro das texturas no banco de dados SQLite local.
  - Opções para equipar, alternar e excluir capas personalizadas diretamente pelo catálogo visual.

- 🕹️ **Novo Visualizador 3D do Personalizador (`skinview3d`):**
  - Renderização 3D idêntica à do Minecraft com iluminação realista, física de caminhada (`WalkingAnimation`), camadas externas completas e exibição de capas com suporte a física.
  - Pausa inteligente de renderização quando fora da tela para poupar GPU e bateria.

- ⏱️ **Painel de Tempo de Jogo em Tempo Real ("YOUR PLAY TIME"):**
  - Gráfico de barras dinâmico dos últimos 7 dias com tooltips interativos e destaque dourado no dia de hoje.
  - Métricas precisas de tempo total dos últimos 7 dias, sessão média, sessão mais longa e dias jogados (`X de 7`), com persistência contínua no SQLite.

- 🗂️ **Organizador de Layout com Física e Animação Fluida (`svelte-dnd-action`):**
  - Reordenação de cards e painéis via arrastar e soltar suave a 60 FPS com transição física de 200ms e confetes de comemoração (`canvas-confetti`).

- 🦀 **Integração das Melhores Bibliotecas do Ecossistema:**
  - Backend Rust potencializado com `rayon` (multi-threading), `moka` (cache concorrente), `fastnbt`, `craftping`, `murmur2`, `notify`, `which`, `sysinfo` e `discord-rich-presence`.
  - Frontend Svelte 5 com `bits-ui`, `@melt-ui/svelte`, `motion`, `howler`, `tone`, `xterm`, `chart.js` e `node-vibrant`.

---

## 🚀 Principais Recursos

### ⚡ Desempenho & Estabilidade
- **Construído em Rust (Tauri 2):** Consumo mínimo de memória e CPU.
- **Pipeline GPU Nativo:** DMA-BUF e aceleração de hardware via WebKitGTK / WebView2.
- **Limitadores de Memória Integrados:** Coleta controlada de assets em páginas pesadas.

### ☕ Java & Otimização
- **Smart Java:** Identifica ou baixa a versão exata do Java (8, 17, 21) correspondente à versão do Minecraft.
- **Aikar's Flags Automáticas:** Aplica parâmetros de JVM para GC suave sem travamentos de frame.
- **Editor de JVM por Instância:** Permite definir memória mínima/máxima e argumentos personalizados.

### 🧩 Mods & Loaders
- **Suporte a Mod Loaders:** Vanilla, Fabric, Forge, NeoForge e Quilt.
- **Central de Mods Integrada:** Busca unificada no Modrinth e CurseForge com filtros por versão e loader.
- **Fallback de Banners:** Exibição elegante com visualização de ícones e banners de modpacks.

### 👕 Personalização
- **Modelo 3D Volumétrico Real:** Rotação 360° interativa, troca entre modelos Classic (4px) e Slim (3px).
- **NameMC:** Busca de skins por nickname.
- **Capas 3D:** 13 capas históricas incluídas e suporte a capas customizadas.

### 🌐 Multiplayer
- **Chat P2P Integrado:** Comunicação direta por IP entre jogadores do Luxmc.

---

## 📥 Como Baixar e Instalar

Baixe o pacote para seu sistema na aba de **[📦 Releases Oficiais (v1.5.6-BETA)](https://github.com/predabr/luxmc/releases/latest)**.

### 🪟 Microsoft Windows (`Luxmc.exe`)
Executável nativo de 64-bit para Windows 10 e Windows 11 com WebView2:
1. Baixe **`Luxmc.exe`** ou o instalador `.msi`.
2. Dê duplo clique para iniciar diretamente.

### 🟢 Universal Linux (AppImage)
Compatível com todas as distribuições Linux:
```bash
# 1. Dê permissão de execução
chmod +x Luxmc_1.5.6-beta_amd64.AppImage

# 2. Execute
./Luxmc_1.5.6-beta_amd64.AppImage
```

### 🟣 Arch Linux / Manjaro
O repositório inclui suporte completo para o Arch Linux:

```bash
# Compilação e instalação nativa via PKGBUILD:
cd packaging/arch
makepkg -si
```

Ou através do pacote de binário pré-compilado:
```bash
cd packaging/aur
makepkg -si
```

### 🔴 Debian / Ubuntu / Pop!_OS (.deb)
```bash
sudo apt install ./Luxmc_1.5.6-beta_amd64.deb
```

### 🔵 Fedora / openSUSE (.rpm)
```bash
sudo dnf install ./Luxmc-1.5.6-beta-1.x86_64.rpm
```

---

## 🛠️ Compilando do Código-Fonte

### Requisitos

| Requisito | Versão Mínima |
|-----------|--------------|
| **Node.js** | v20+ |
| **pnpm** | v9+ |
| **Rust** (stable) | 1.77+ |
| **WebKitGTK** (Linux) | 4.1+ |
| **GTK** (Linux) | 3.0+ |

### Compilação

```bash
# 1. Clone o repositório
git clone https://github.com/predabr/luxmc.git
cd luxmc

# 2. Instale as dependências
pnpm install

# 3. Inicie em desenvolvimento
pnpm tauri dev

# 4. Gere os pacotes de produção
pnpm tauri build
```

---

## 🏗️ Estrutura do Projeto

```
luxmc/
├── src/                          # Frontend (SvelteKit + Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── api/                  # Wrappers tipados para Tauri commands
│   │   ├── components/           # Componentes UI (Skins, Servidores, Mods, Cutscene)
│   │   ├── stores/               # Estado reativo (Svelte 5 runes)
│   │   └── utils/                # Texturas e utilitários
│   └── routes/                   # Páginas SvelteKit (Dashboard, Mods, Skins, Servidores, Amigos)
├── src-tauri/                    # Backend Rust (Tauri 2)
│   ├── src/
│   │   ├── core/                 # Lógica de negócio (Java, Downloader, Minecraft, P2P)
│   │   ├── commands/             # Handlers #[tauri::command]
│   │   └── lib.rs                # Registro de plugins e comandos
│   └── Cargo.toml
├── packaging/
│   ├── arch/                     # PKGBUILD nativo de compilação para Arch Linux
│   └── aur/                      # PKGBUILD binário para AUR
└── static/                       # Assets estáticos
```

---

## 🤝 Contribuindo

1. **Fork** o repositório
2. Crie sua branch de funcionalidade: `git checkout -b feat/minha-feature`
3. Execute as verificações do projeto antes de submeter:
   ```bash
   pnpm check       # Verificação de tipos Svelte/TS
   cargo check      # Verificação de compilação Rust (dentro de src-tauri/)
   ```
4. Abra um Pull Request.

---

## 📄 Licença

Distribuído sob a **Licença Própria do Luxmc Launcher** (Copyright © 2026 Pedro & Time Luxmc).
Gratuito para uso pessoal e não comercial. Consulte [LICENSE](LICENSE) para detalhes.

<div align="center">
  <br/>
  <p><strong>Feito com dedicação para a comunidade gamer.</strong></p>
  <img src="static/logo.png" alt="Luxmc" width="60" />
</div>
