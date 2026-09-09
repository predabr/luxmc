<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="200" />
  <h1>Luxmc Launcher <span style="color:#e2b86b">v0.5 BETA</span></h1>
  <p><strong>Um launcher de Minecraft focado em performance, feito de Linux para Linux.</strong></p>
</div>

<br/>

<div align="center">
  <img src="static/home.png" alt="Home Screen" style="border-radius: 12px; margin-bottom: 20px;" width="800" />
</div>

<div align="center" style="display: flex; justify-content: center; gap: 20px;">
  <img src="static/servers.png" alt="Servers List" style="border-radius: 12px; width: 48%;" />
  <img src="static/skins.png" alt="3D Skin Customization" style="border-radius: 12px; width: 48%;" />
</div>

<br/>

## 🚀 O que é o Luxmc?
O Luxmc é um launcher moderno construído com **Rust (Tauri)** e **SvelteKit**. Ele foi desenhado para extrair o máximo de performance no Linux, com integração nativa ao sistema, baixo uso de RAM e um visual de cair o queixo.

- ⚡ **Extremamente Leve:** Backend nativo em Rust (baixo uso de recursos).
- 🧠 **Smart RAM & Otimização:** Calcula automaticamente a melhor proporção de memória e injeta as *Aikar's Flags* para o Minecraft rodar liso.
- 🐧 **Linux-first:** Suporte nativo ao Vulkan (Zero-Lag) e otimizado para distros Linux.
- 🎨 **Visual 3D:** Cutscene em tempo real, visualizador volumétrico de skins e design premium de vidro fosco.
- 🌐 **Host de Mundo P2P:** Jogue com amigos abrindo seu mundo pra LAN/Internet direto pelo launcher!
- 🗂 **Modpacks:** Explorador de modpacks focado na sua diversão.
- 📥 **Atualizações Silenciosas:** O launcher verifica novas versões do motor e notifica com estilo.

---

## 📥 Como Baixar e Instalar

Você pode baixar a versão mais recente na nossa página de **[Releases](https://github.com/predabr/luxmc/releases)**.

### 🟢 Universal Linux (AppImage)
Baixe o arquivo `.AppImage`, permita a execução e abra:
```bash
chmod +x luxmc-0.5.0-beta.AppImage
./luxmc-0.5.0-beta.AppImage
```

### 🔴 Debian-based (Ubuntu, Pop!_OS, Mint)
```bash
sudo apt install ./luxmc_0.5.0_amd64.deb
```

### 🔵 RedHat-based (Fedora, openSUSE)
```bash
sudo dnf install ./luxmc-0.5.0.x86_64.rpm
```

### 🟣 Arch Linux (AUR)
```bash
yay -S luxmc-bin
```

---

## 🛠️ Desenvolvedores (Como Compilar)
1. Clone o repositório: `git clone https://github.com/predabr/luxmc.git`
2. Instale as dependências: `pnpm install`
3. Inicie o modo Dev: `pnpm tauri dev`
