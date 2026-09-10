<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="180" />
  <h1>Luxmc Launcher <span style="color:#e2b86b">v1.0.0-BETA</span></h1>
  <p><strong>O Minecraft Launcher de alta performance projetado de Linux para Linux.</strong></p>

  <p>
    <a href="https://github.com/predabr/luxmc/releases"><img src="https://img.shields.io/badge/version-v1.0.0--BETA-gold?style=for-the-badge&logo=rocket" alt="Version" /></a>
    <img src="https://img.shields.io/badge/platform-Linux-blue?style=for-the-badge&logo=linux" alt="Linux" />
    <img src="https://img.shields.io/badge/Windows-Em%20Breve-0078D4?style=for-the-badge&logo=windows" alt="Windows Em Breve" />
    <img src="https://img.shields.io/badge/backend-Rust%20%2F%20Tauri%202-orange?style=for-the-badge&logo=rust" alt="Rust Tauri 2" />
    <img src="https://img.shields.io/badge/frontend-Svelte%205-red?style=for-the-badge&logo=svelte" alt="Svelte 5" />
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-Luxmc%20License-purple?style=for-the-badge" alt="License" /></a>
  </p>
</div>

<br/>

<div align="center">
  <img src="static/home.png" alt="Home Screen" style="border-radius: 14px; margin-bottom: 20px; box-shadow: 0 20px 50px rgba(0,0,0,0.5);" width="850" />
</div>

<div align="center" style="display: flex; justify-content: center; gap: 20px;">
  <img src="static/servers.png" alt="Servers List" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
  <img src="static/skins.png" alt="3D Skin Customization" style="border-radius: 12px; width: 48%; box-shadow: 0 10px 30px rgba(0,0,0,0.4);" />
</div>

<br/>

## 🌟 O que há de novo na v1.0.0-BETA?

- ⚡ **Zero-Lag & Aceleração GPU Total:** Pipeline DMA-BUF ativo no WebKitGTK para o Linux AppImage rodar a 60–144 FPS liso na sua GPU (Mesa / Vulkan / OpenGL), sem software rasterization por CPU.
- 📦 **Central de Mods com Visão Detalhada, Galeria e Versões:** Navegue com descrição formatada, galeria de screenshots com lightbox, changelogs e downloads de versões específicas direto do Modrinth e CurseForge.
- 🧩 **Gerenciador Interno de Mods por Instância:** Ative ou desative mods instantaneamente (`.jar` ↔ `.jar.disabled`), exclua, adicione arquivos locais `.jar` ou abra a pasta nativa no seu gerenciador de arquivos Linux com 1 clique.
- 🚀 **Predefinições Rápidas de Instâncias em 1 Clique:** Comece a jogar em segundos com presets prontos de *Vanilla Otimizado (1.21.4 Fabric)*, *PvP Clássico (1.8.9 Vanilla)*, *Survival Moderno* e *Modded NeoForge*.
- 📋 **Compartilhamento de Logs e Crash Reports via `mclo.gs`:** Envie relatórios de erro e logs do jogo com sanitização de dados sensíveis e obtenha links compartilháveis com cópia automática para a área de transferência.
- 🔑 **Suporte Dinâmico a Microsoft Azure Client ID:** Configure seu próprio Client ID de autenticação Microsoft diretamente na tela de Configurações sem precisar recompilar.
- 🎬 **Nova Cutscene Cinematográfica:** Abertura rápida de 2.3s com anéis orbitais, snap magnético e efeito sonoro sintetizado em tempo real (pule instantaneamente com `Espaço` ou clique).
- 🎨 **Redesign Completo da Interface:**
  - Sidebar em squircle com gradiente metálico refinado e alternador rápido de contas no topo.
  - Switches de alternância elegantes no estilo das configurações modernas (trilha champanhe ativa e thumb de alto contraste).
  - Central de Mods com badges de fonte, nuvens de tags de categorias e pills de filtros.
  - Dashboard inicial com saudação, atalho para bibliotecas, lista dos servidores mais jogados e monitoramento do seu tempo de jogo semanal.

---

## 🚀 Principais Recursos

- ⚡ **Construído em Rust (Tauri 2):** Consumo mínimo de memória RAM e inicialização instantânea.
- ☕ **Smart Java & Auto-Otimização:** Detecta ou baixa o Java correto para cada versão do Minecraft e aplica automaticamente as melhores *Aikar's Flags*.
- 🧩 **Suporte a Múltiplos Mod Loaders:** Vanilla, Fabric, Forge, NeoForge e Quilt com isolamento seguro de instâncias.
- 👕 **Personalização 3D de Skins & Capas:** Renderizador 3D volumétrico em tempo real, busca de skins via NameMC e suporte a capas OptiFine, Mojang e Migrator.
- 🌐 **Mundo Aberto P2P:** Jogue mundos locais com seus amigos via rede/LAN diretamente pelo launcher.
- 🔒 **Privacidade & Segurança Linux:** Tokens de sessão protegidos pelo Keyring seguro do sistema (libsecret / KWallet) e opção de modo anônimo para streamers.
- 📥 **Atualizações Automáticas:** Verificação e alertas nativos para novas versões do launcher.

---

## 📥 Como Baixar e Instalar

Baixe o pacote correspondente à sua distribuição na aba de **[Releases Oficiais](https://github.com/predabr/luxmc/releases)**.

### 🟢 Universal Linux (AppImage)
Recomendado para qualquer distribuição Linux (Ubuntu, Debian, Fedora, Arch, Pop!_OS, openSUSE, Mint, etc.):
```bash
chmod +x luxmc-1.0.0-beta.AppImage
./luxmc-1.0.0-beta.AppImage
```

### 🔴 Debian / Ubuntu / Pop!_OS / Linux Mint (.deb)
```bash
sudo apt install ./luxmc_1.0.0_amd64.deb
```

### 🔵 Fedora / RHEL / openSUSE (.rpm)
```bash
sudo dnf install ./luxmc-1.0.0.x86_64.rpm
```

### 🟣 Arch Linux / Manjaro (AUR)
```bash
yay -S luxmc-bin
# ou
paru -S luxmc-bin
```

### 🪟 Microsoft Windows (Em Breve)
> 🚀 **Roadmap Ativo:** O suporte nativo ao **Windows** (instaladores oficiais `.msi` e `.exe`) está atualmente em fase de finalização e será disponibilizado nas próximas atualizações do Luxmc Launcher!

---

## 🛠️ Como Compilar do Código-Fonte

### Pré-requisitos
- **Node.js** (v20+) e **pnpm** (v9+)
- **Rust** e Cargo (stable)
- Bibliotecas de desenvolvimento do WebKitGTK e GTK3:
  ```bash
  # Debian/Ubuntu:
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
  
  # Arch Linux:
  sudo pacman -S webkit2gtk-4.1 base-devel openssl gtk3 libappindicator-gtk3 librsvg
  
  # Fedora:
  sudo dnf install webkit2gtk4.1-devel @development-tools openssl-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
  ```

### Compilando e Rodando

```bash
# 1. Clone o repositório
git clone https://github.com/predabr/luxmc.git
cd luxmc

# 2. Instale as dependências frontend
pnpm install

# 3. Inicie em modo de desenvolvimento
pnpm tauri dev

# 4. Gere os pacotes de produção (AppImage, deb, etc.)
pnpm tauri build
```

---

## 📄 Licença

Distribuído sob a **Licença Própria do Luxmc Launcher** (Copyright © 2026 Pedro & Time Luxmc). 
O software é gratuito para uso pessoal e não comercial. É expressamente proibida a revenda, comercialização, empacotamento com anúncios (adware) ou rebranding não autorizado. Para ler todos os termos e condições, consulte o arquivo [LICENSE](LICENSE).
