<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="180" />
  <h1>Luxmc Launcher <span style="color:#e2b86b">v1.3.1-ALPHA</span></h1>
  <p><strong>O Minecraft Launcher moderno, leve e de alta performance.</strong></p>
  <p><em>Rápido. Moderno. Poderoso. 100% gratuito.</em></p>

  <p>
    <a href="https://github.com/predabr/luxmc/releases"><img src="https://img.shields.io/badge/version-v1.3.1--ALPHA-gold?style=for-the-badge&logo=rocket" alt="Version" /></a>
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
  <img src="static/servers.png" alt="Diretório de Servidores com ping em tempo real" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
  <img src="static/skins.png" alt="Personalização 3D de Skins & Capas" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
</div>
<div align="center">
  <p><em>Esquerda: Diretório de 100+ servidores com paginação fluida e economia de RAM. Direita: Personalização 3D volumétrica com catálogo HD e importação de capas .PNG.</em></p>
</div>

<br/>

---

## 🎮 O que é o Luxmc?

O **Luxmc** é um launcher de Minecraft moderno construído **do zero em Rust (Tauri 2)** com frontend em **Svelte 5 + TypeScript + Tailwind CSS**. Projetado com foco primário em estabilidade no Linux e suporte nativo ao Windows, ele oferece consumo reduzido de RAM, inicialização ultra-rápida e uma experiência premium inspirada nos melhores launchers modernos.

### Por que usar o Luxmc?

| Recurso | Luxmc | Launcher Oficial | Outros Launchers |
|---------|-------|-----------------|-------------------|
| **Consumo de RAM** | ~180-220 MB | ~800 MB+ | ~400-600 MB |
| **Inicialização** | < 2s | ~8-15s | ~5-10s |
| **Central de Mods** | ✅ Modrinth + CurseForge | ❌ | ✅ Parcial |
| **Personalização 3D** | ✅ Volumétrico real + 13 Capas | ❌ | ✅ 2D apenas |
| **Diretório de Servidores** | ✅ 100+ com ping paginado | ❌ | ❌ |
| **Chat & P2P Multiplayer** | ✅ Socket TCP Direto | ❌ | ❌ |
| **Nativo Linux & Windows** | ✅ Tauri 2 / Rust | ❌ Electron | ❌ Java/Electron |

---

## 🌟 Novidades na v1.3.1-ALPHA

Esta versão foca na resolução de problemas críticos de execução no Windows, compatibilidade retroativa de bibliotecas legadas, otimização extrema de memória e expansão da personalização:

- 🪟 **Correção Definitiva do Erro Win32 193 no Windows:**
  - Identificação de plataforma em runtime (`windows-x64`) no gerenciador de downloads do Java da Mojang, baixando binários nativos executáveis PE (`java.exe`) em vez de binários ELF incompatíveis.
  - Inicialização garantida de qualquer versão (Vanilla, Fabric, Forge, NeoForge, Quilt) no Windows 10/11.
- 📦 **Suporte a Bibliotecas com Classificadores e Fallback BMCLAPI (Fim do 404):**
  - Download e descompactação de bibliotecas nativas via `downloads.classifiers` (`natives-windows`, `natives-linux`, `natives-osx`).
  - Resolução de URLs de bibliotecas legadas (como bibliotecas Twitch de versões 1.7/1.8 que a Mojang removeu de seus servidores) através de espelho com tolerância a falhas sem abortar a inicialização.
- 🔑 **Autenticação Microsoft Corrigida (`/consumers/`):**
  - Resolução do erro `AADSTS9002346` redirecionando o fluxo OAuth para contas pessoais Microsoft (`login.microsoftonline.com/consumers/...`).
  - Servidor de callback local aprimorado com tela dark temática informando o status da autorização.
- 🛡️ **Importação de Capas Separada & Catálogo HD Completo:**
  - Botão dedicado **"Importar Capa (.PNG)"** que adiciona e equipa capas personalizadas sem sobrescrever a skin do jogador.
  - Catálogo de **13 capas oficiais e comemorativas** renderizadas em pixel-art HD e aplicadas em tempo real com física e rotação no modelo 3D.
- 🧠 **Otimização de RAM no Diretório de Servidores (WebKitGTK & AppImage):**
  - Sistema de paginação (20 servidores por página) com ping sob demanda restrito aos servidores em exibição.
  - Carregamento de imagens com `loading="lazy"` e `decoding="async"`, eliminando sobrecargas de memória no Linux AppImage.
- 💬 **Chat P2P em Tempo Real com Reatividade Svelte 5:**
  - Adicionado timeout de 3 segundos em transmissões TCP para evitar congelamento de interface com hosts inativos.
  - Atualização reativa instantânea de mensagens de chat no Svelte 5 sem necessidade de recarregar a tela.
- 🎬 **Nova Cutscene 3D de Abertura:**
  - Animação conceitual onde a logo se divide em 4 quadrantes, separa-se pelas extremidades, realiza um giro completo de 360°, conecta-se novamente ao centro com impacto suave e aciona um som harmônico de cristal sintetizado nativamente via **Web Audio API**.
- 🐧 **Suporte Nativo ao Arch Linux:**
  - Arquivos oficiais `PKGBUILD` prontos para empacotamento local e AUR.

---

## 🚀 Principais Recursos

### ⚡ Desempenho & Estabilidade
- **Construído em Rust (Tauri 2):** Consumo mínimo de memória e CPU.
- **Pipeline GPU nativo:** DMA-BUF e aceleração de hardware via WebKitGTK / WebView2.
- **Limitadores de memória integrados:** Coleta controlada de assets em páginas pesadas.

### ☕ Java & Otimização
- **Smart Java:** Identifica ou baixa a versão exata do Java (8, 17, 21) correspondente à versão do Minecraft e à plataforma do usuário.
- **Aikar's Flags automáticas:** Aplica parâmetros de JVM para GC suave sem travamentos de frame.
- **Editor de JVM por instância:** Permite definir memória mínima/máxima e argumentos personalizados.

### 🧩 Mods & Loaders
- **Suporte a Mod Loaders:** Vanilla, Fabric, Forge, NeoForge e Quilt.
- **Central de Mods Integrada:** Busca unificada no Modrinth e CurseForge com filtros por versão e loader.
- **Fallback de Banners:** Exibição elegante com visualização de ícones e banners de modpacks.

### 👕 Personalização
- **Modelo 3D Volumétrico Real:** Rotação 360° interativa, troca entre modelos Classic (4px) e Slim (3px).
- **NameMC:** Busca de skins por nickname.
- **Capas 3D:** 13 capas históricas incluídas e suporte a capas customizadas.

### 🌐 Servidores & Multiplayer
- **Diretório de 100+ Servidores:** Brasileiros e mundiais com ping em tempo real e paginação econômica.
- **Chat P2P Integrado:** Comunicação direta por IP entre jogadores do Luxmc.

---

## 📥 Como Baixar e Instalar

Baixe o pacote para seu sistema na aba de **[📦 Releases Oficiais (v1.3.1-ALPHA)](https://github.com/predabr/luxmc/releases/latest)**.

### 🪟 Microsoft Windows (`Luxmc.exe`)
Executável nativo de 64-bit para Windows 10 e Windows 11 com WebView2:
1. Baixe **`Luxmc.exe`** ou o instalador `.msi`.
2. Dê duplo clique para iniciar diretamente.

### 🟢 Universal Linux (AppImage)
Compatível com todas as distribuições Linux:
```bash
# 1. Dê permissão de execução
chmod +x Luxmc_1.3.1-alpha_amd64.AppImage

# 2. Execute
./Luxmc_1.3.1-alpha_amd64.AppImage
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
sudo apt install ./Luxmc_1.3.1-alpha_amd64.deb
```

### 🔵 Fedora / openSUSE (.rpm)
```bash
sudo dnf install ./Luxmc-1.3.1-alpha-1.x86_64.rpm
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
