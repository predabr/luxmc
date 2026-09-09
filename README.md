<div align="center">
  <img src="static/logo.png" alt="Luxmc Logo" width="250" />
  <h1>Luxmc Launcher</h1>
  <p><strong>Um launcher de Minecraft focado em performance, feito de Linux para Linux.</strong></p>
</div>

<div align="center">
  <img src="static/screenshot.png" alt="Luxmc UI" style="border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.5);" />
</div>

## 🚀 O que é o Luxmc?
O Luxmc é um launcher moderno construído com **Rust (Tauri)** e **SvelteKit**. Ele foi desenhado para extrair o máximo de performance no Linux, com integração nativa ao sistema, baixo uso de RAM e um visual de cair o queixo.

- ⚡ **Extremamente Leve:** Backend nativo em Rust (baixo uso de recursos).
- 🧠 **Smart RAM & Otimização:** Calcula automaticamente 50% da sua memória RAM e injeta as melhores flags JVM de performance (estilo Aikar's) para o Minecraft rodar liso, sem você precisar configurar nada!
- 🐧 **Linux-first:** Sem bugs estranhos, feito para rodar liso no seu pinguim (com suporte inteligente a Vulkan).
- 🎨 **Customizável:** Cores de destaque dinâmicas, temas e uma cutscene de abertura 3D com físicas reais.
- 🌐 **Host de Mundo:** Jogue com amigos abrindo seu mundo pra LAN/Internet direto pelo launcher!
- 🗂 **Modpacks & Instâncias:** Gerenciador completo de instâncias.
- 📥 **Update Checker:** O launcher avisa de forma inteligente (com uma UI super bonita) quando há uma nova atualização disponível!

---

## 📥 Como Baixar e Instalar

Você pode baixar a versão mais recente do Luxmc (v0.2.0) na nossa página oficial de **[Releases (Lançamentos)](https://github.com/predabr/luxmc/releases)**. Escolha o arquivo de acordo com a sua distribuição Linux:

### 🟢 Qualquer Linux (AppImage)
Baixe o arquivo `.AppImage`, clique com o botão direito, vá em propriedades, marque a opção "Permitir execução como programa" e dê dois cliques para abrir!
*(Se preferir pelo terminal):*
```bash
chmod +x luxmc-0.2.0.AppImage
./luxmc-0.2.0.AppImage
```

### 🔴 Ubuntu, Pop!_OS, Debian, Mint (.deb)
Baixe o arquivo `.deb` e dê dois cliques para instalar pela loja do seu sistema, ou use o terminal:
```bash
sudo apt install ./luxmc_0.2.0_amd64.deb
```

### 🔵 Fedora, openSUSE (.rpm)
Baixe o arquivo `.rpm` e dê dois cliques, ou instale pelo terminal:
```bash
sudo dnf install ./luxmc-0.2.0.x86_64.rpm
```

### 🟣 Arch Linux, Manjaro, EndeavourOS (AUR)
*(Em breve disponível no repositório da comunidade)*
```bash
yay -S luxmc-bin
```

---

## 🛠️ Desenvolvedores (Como Compilar)
Se você quer ajudar a desenvolver o Luxmc ou testar a versão beta:

1. Clone o repositório: `git clone https://github.com/predabr/luxmc.git`
2. Entre na pasta: `cd luxmc`
3. Instale as dependências: `pnpm install`
4. Inicie o modo Dev: `pnpm tauri dev`
