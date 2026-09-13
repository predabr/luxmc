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

## 🌟 Novidades na v1.5.5-BETA

- 📦 **Motor de Download CurseForge com Fallback em 4 Níveis:**
  - Resolução robusta de modpacks que antes ficavam com mods faltando (ex: All The Mods 10/11).
  - Cascata com API individual de arquivos, endpoint em lote, Edge CDN e mirrors de contingência.
  - Validação de integridade de arquivos `.jar` garantindo que arquivos truncados sejam descartados antes da inicialização.
  - **Botão de Cancelar Download**: permite cancelar a instalação de modpacks em andamento a qualquer momento com um clique.

- 🧠 **Launcher Inteligente (Auto-Hardware & Alocação Dinâmica):**
  - O launcher analisa dinamicamente os recursos da máquina (RAM total e GPU).
  - Em modpacks pesados (Forge e NeoForge com centenas de mods), eleva automaticamente a memória alocada para 6 GB a 8 GB (ou 10 GB em sistemas com 24 GB+), prevenindo travamentos por OutOfMemory e lentidão no carregamento.

- 🛠️ **Correção Definitiva de Classpath no NeoForge:**
  - Resolução do crash de conflito de módulos da JVM (`Modules minecraft and _1_21_1 both export...`) através da deduplicação inteligente do jar vanilla quando o NeoForge já fornece o cliente embutido.

- 👕 **Personalizador de Skins 3D Aprimorado:**
  - Sincronização persistente que não sobrescreve a escolha do usuário na inicialização.
  - Injeção automática e nativa da skin selecionada no jogo via resource pack do Luxmc (funciona em contas Microsoft e contas locais).
  - Suporte resiliente com espelhos de textura e alternância instantânea entre modelos Classic (4px) e Slim (3px).

- 📋 **Modal de Instâncias Centralizado e Responsivo:**
  - Novo modal overlay (`CreateInstanceModal`) fixo e centralizado na tela, com suporte para fechar com tecla `Escape` ou clicando fora.
  - Cabeçalho com efeito blur e fixação `sticky`, mantendo as ações sempre acessíveis.

- 🎨 **Organizador de Layout Modular (`/organizer`):**
  - Permite reorganizar os cards e widgets da tela inicial via arrastar e soltar (drag-and-drop), ajustar larguras e ocultar seções com salvamento automático local.

- 🚀 **Animações e Efeitos Fluidos:**
  - Animação de transição lateral ao alternar entre abas.
  - Efeitos táteis de iluminação (`.animate-pulse-glow`) e shimmer nos cards e botões.

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

Baixe o pacote para seu sistema na aba de **[📦 Releases Oficiais (v1.5.5-BETA)](https://github.com/predabr/luxmc/releases/latest)**.

### 🪟 Microsoft Windows (`Luxmc.exe`)
Executável nativo de 64-bit para Windows 10 e Windows 11 com WebView2:
1. Baixe **`Luxmc.exe`** ou o instalador `.msi`.
2. Dê duplo clique para iniciar diretamente.

### 🟢 Universal Linux (AppImage)
Compatível com todas as distribuições Linux:
```bash
# 1. Dê permissão de execução
chmod +x Luxmc_1.5.5-beta_amd64.AppImage

# 2. Execute
./Luxmc_1.5.5-beta_amd64.AppImage
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
sudo apt install ./Luxmc_1.5.5-beta_amd64.deb
```

### 🔵 Fedora / openSUSE (.rpm)
```bash
sudo dnf install ./Luxmc-1.5.5-beta-1.x86_64.rpm
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
