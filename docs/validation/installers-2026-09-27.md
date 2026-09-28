# Luxmc: auditoria dos instaladores e inicialização

## Alterações

- `packaging/arch/PKGBUILD` usa `pnpm build` e `pnpm tauri build --no-bundle -- --locked`. O binário de produção recusa compilação sem o protocolo de assets do Tauri. `frontendDist`, `beforeBuildCommand`, URL `/` e adapter-static foram conferidos.
- `scripts/build-arch-package.sh` recompila pelo Tauri para o alvo Linux explícito, resolve o diretório real pelo Cargo metadata e valida arquivo não vazio, executável e ELF x86-64. Não aceita debug ou escolhe um release antigo entre vários diretórios. A versão vem de `tauri.conf.json`; as oito dependências Arch solicitadas estão no `.PKGINFO`.
- AppImage restringe ambos os caminhos GStreamer, versionados e legados, às pastas do AppDir. Reconhece layouts lib, lib64 e multiarch, além do scanner em libexec. Se faltar scanner, não herda o scanner do host: mantém caminho interno e emite diagnóstico. Nenhuma decisão depende da versão GStreamer do host. A presença real dos plugins continua sendo responsabilidade do pacote.
- Minecraft mantém as quatro variáveis X11 e o argumento LWJGL existentes. Sem DISPLAY, informa que XWayland é necessário, em vez de tentar novamente o caminho Wayland incompatível.
- Windows cria a chave `UserGpuPreferences` quando ausente e grava `GpuPreference=2;` para o Java executado. Remove do subprocesso o shim legado e variáveis antigas de OpenCL herdadas.
- A identidade GLib `luxmc`/`Luxmc` já estava definida antes da inicialização. O registro automático do plugin de deep links já estava restrito ao Windows. DEB e RPM passam a usar o mesmo conteúdo de desktop do Arch com `StartupWMClass=luxmc` e `x-scheme-handler/luxmc`. O Tauri nomeia o arquivo desses pacotes `Luxmc.desktop`; a integração reconhece esse nome e `luxmc.desktop` do Arch, sem criar outra entrada de usuário quando já existe uma entrada instalada pelo pacote.
- DEB usa dependência Ayatana compatível com o build; RPM declara a integração Ayatana, ícones e xdg-utils.
- Instalador tenta pacote nativo em Arch, Debian/Ubuntu e Fedora. Resolve nomes dos assets da release; downloads usam diretório temporário exclusivo e limpeza por trap. Pacman recebe caminho local. APT instala o DEB com resolução de dependências, sem declarar sucesso apenas porque `apt-get -f` terminou. RPM usa DNF. AppImage só substitui o executável anterior após download concluído. Uma instalação nativa bem-sucedida remove o AppImage de usuário que poderia esconder `/usr/bin/luxmc` no PATH.
- Desinstalador remove pacotes das três famílias, executáveis locais, atalhos e ícones de usuário e do sistema, usando elevação apenas quando necessária. Atualiza os caches e propaga falhas. Mundos, instâncias e dados pessoais são preservados.
- CI Linux e Windows executam verificações antes do empacotamento. A orientação de instalação Arch na release baixa para `/tmp` antes de chamar pacman.

## Verificação local

- `pnpm check`: zero erros e zero avisos.
- `pnpm build`: assets estáticos gerados.
- `cargo test --lib`: 72 passaram, zero falhas, um ignorado.
- `cargo check`: aprovado.
- `python3 tests/installers.test.py`: cinco testes aprovados, incluindo subcasos Arch/Debian/Fedora, fallback, download incompleto e desinstalação sem apagar mundo de teste. Gerenciadores de pacotes são simulados; nenhum pacote do host foi instalado ou removido.
- `bash -n` e `git diff --check`: aprovados.
- Metadados Cargo do Tauri em release: feature `custom-protocol` confirmada.

## Procedimento de reprodução

```sh
pnpm check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo check --manifest-path src-tauri/Cargo.toml
python3 tests/installers.test.py
bash scripts/build-arch-package.sh
pnpm tauri bundle --target x86_64-unknown-linux-gnu --bundles deb,rpm,appimage
```

## Artefatos e abertura local

O release final produzido pelo Tauri mediu 45.264.768 bytes (43,17 MiB). O pacote Arch final com propriedade root:root mediu 18.685.422 bytes (17,82 MiB). O SHA-256 do executável nativo coincide com o extraído do pacote: `d0359692f8cfe4b774c90b91812d9af3769963bec896a20fd6344d54f72e8cc8`. O executável extraído do pacote abriu a tela de login em sessão D-Bus e diretórios XDG isolados, com classe de janela `luxmc` e sem listener na porta 1420. A captura está em [luxmc-release-login.png](luxmc-release-login.png).

A inspeção do DEB confirmou nome de pacote `luxmc`, dependências e arquivo `usr/share/applications/Luxmc.desktop` com a classe e protocolo esperados. DEB/RPM foram gerados, mas não instalados em sistemas Debian/Fedora neste teste.

No Arch atual, o plugin GTK externo do linuxdeploy falhou ao gerar `loaders.cache`: o GDK-Pixbuf usa loaders embutidos e o diretório esperado pelo script não existe. Criar o diretório de cache no AppDir de teste permitiu avançar sem modificar arquivos do host ou o plugin. Não se deve confundir esse contorno local com uma validação do comando padrão em todas as distribuições. A CI usa Ubuntu 22.04. Docker está instalado, mas seu daemon não está disponível neste host.

O AppImage final abriu o frontend embutido: [captura da revisão final](luxmc-appimage-final.png). A captura ocorreu durante a apresentação inicial. O teste anterior, com o mesmo código de inicialização, chegou à tela de login. Os processos de teste foram encerrados deliberadamente por `timeout` (código 124).

O AppImage de teste abriu a tela de login no Hyprland deste host, sem servidor Vite e sem ativar renderização por software: [captura](luxmc-appimage-login.png). Foi executado com `APPIMAGE_EXTRACT_AND_RUN=1`, portanto o caminho de montagem FUSE não foi validado. Há avisos de acessibilidade, nome GLib definido duas vezes e detecção de AppImage em modo extraído; não houve SIGABRT ou falha de autoaudiosink observada.

O AppDir contém `usr/lib/gstreamer-1.0/libgstautodetect.so` e `usr/lib/gstreamer-1.0/gst-plugin-scanner`, caminho acrescentado à resolução do scanner nesta correção. `gst-inspect-1.0 autoaudiosink`, com bibliotecas, caminhos de plugins e scanner apontados ao AppDir e cache temporário, encontrou o elemento no plugin empacotado, versão 1.28.7. Essa inspeção usa o executável gst-inspect do host.

Tamanhos finais: executável 45.264.768 bytes; Arch 18.685.422; DEB 23.025.770; RPM 23.025.875; AppImage 185.510.392. Caminhos e hashes de todos os artefatos: [manifesto](installers-artifacts.json).

O tamanho aproximado de 44 MB/18 MB é uma referência, não prova de frontend embutido: compressão, toolchain e assets alteram esses valores. A procedência do build pelo Tauri e a inicialização do artefato são as verificações relevantes.

## Validação externa necessária

Não há garantia de funcionamento em 100% das combinações de SO e driver a partir de um host Arch. Antes de publicar, testar instalação, atualização, `luxmc://`, inicialização sem servidor Vite e desinstalação em Arch, Ubuntu/Debian e Fedora limpos; AppImage com GStreamer 1.24+ e XWayland; Windows 10/11 com NSIS e Java 21/NeoForge/ATM11 em GPUs Intel, AMD e NVIDIA. Confirmar no Windows a chave de GPU e ausência de WGL 0x10008. Drivers sem suporte ao OpenGL exigido pelo Minecraft não são corrigidos por uma preferência de GPU.

O binário gerado neste Arch depende da glibc local; para distribuir DEB/RPM/AppImage, usar o ambiente Ubuntu da CI e verificar compatibilidade nas distribuições mínimas suportadas.

## Referências

- [Build e distribuição Tauri](https://v2.tauri.app/distribute/)
- [Configuração Tauri](https://v2.tauri.app/reference/config/)
- [Busca de plugins GStreamer](https://gstreamer.freedesktop.org/documentation/gstreamer/running.html)

## Contorno local do linuxdeploy no Arch

Após a falha específica de criação de `loaders.cache` descrita acima, o AppDir gerado pelo Tauri já contém o executável e as dependências. Para reproduzir o contorno usado no teste, a partir da raiz do projeto:

```sh
appdir="$PWD/src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/appimage/Luxmc.AppDir"
mkdir -p "$appdir/usr/lib/gdk-pixbuf-2.0/2.10.0"
DEPLOY_GTK_VERSION=3 ARCH=x86_64 \
  OUTPUT="$PWD/src-tauri/target/x86_64-unknown-linux-gnu/release/bundle/appimage/Luxmc_2.0.0_amd64.AppImage" \
  "$HOME/.cache/tauri/linuxdeploy-x86_64.AppImage" --appimage-extract-and-run \
  --appdir "$appdir" --plugin gtk --plugin gstreamer --output appimage
```

Esse comando retoma o empacotamento; não compila o Rust nem substitui o build pelo Tauri. Não altera o plugin externo ou os diretórios de bibliotecas do sistema. O build público continua devendo sair do ambiente Ubuntu da CI.
