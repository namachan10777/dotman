# Dotman

[![Rust](https://github.com/namachan10777/dotman/actions/workflows/rust.yml/badge.svg)](https://github.com/namachan10777/dotman/actions/workflows/rust.yml)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

```yaml
---
taskgroups:
  rust_unix_common:
  - type: sh
    cmd: ["sh", "-c", "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -- -y"]
    test: "{{env.HOME}}/.cargo/bin/rustup"
    sha256:
      sakanainu: 3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338
  - { type: cargo, package: alacritty }
  - { type: cargo, package: bandwhich }
  - { type: cargo, package: bat }
  - { type: cargo, package: battop }
  - { type: cargo, package: bingrep }
  - { type: cargo, package: cargo-edit }
  - { type: cargo, package: cargo-update }
  - { type: cargo, package: cross }
  - { type: cargo, package: csview }
  - { type: cargo, package: diskonaut }
  - { type: cargo, package: fd-find }
  - { type: cargo, package: git-delta }
  - { type: cargo, package: git-interactive-rebase-tool }
  - { type: cargo, package: gping }
  - { type: cargo, package: ht }
  - { type: cargo, package: hyperfine }
  - { type: cargo, package: lsd }
  - { type: cargo, package: onefetch }
  - { type: cargo, package: pastel }
  - { type: cargo, package: procs }
  - { type: cargo, package: ripgrep }
  - { type: cargo, package: silicon }
  - { type: cargo, package: skim }
  - { type: cargo, package: starship }
  - { type: cargo, package: tokei }
  - { type: cargo, package: topgrade }
  - { type: cargo, package: xsv }
  - { type: cargo, package: zoxide }

  unix_common:
  - { type: cp, src: pkgs/fish,              dest: "{{env.XDG_CONFIG_HOME}}/fish" }
  - { type: cp, src: pkgs/gpg,               dest: "{{env.HOME}}/.gnupg/" }
  - { type: cp, src: pkgs/latexmk/latexmkrc, dest: "{{env.HOME}}/.latexmkrc" }
  - { type: cp, src: pkgs/lazygit,           dest: "{{env.XDG_CONFIG_HOME}}/jesseduffield/lazygit" }
  - { type: cp, src: pkgs/neovim,            dest: "{{env.XDG_CONFIG_HOME}}/neovim" }
  - { type: cp, src: pkgs/npm/npmrc,         dest: "{{env.HOME}}/.npmrc" }
  - { type: cp, src: pkgs/tig/tigrc,         dest: "{{env.HOME}}/.tigrc" }
  - type: cp
    src: "pkgs/alacritty/"
    dest: "{{env.XDG_CONFIG_HOME}}/alacritty/"
    templates:
      - target: pkgs/alacritty/alacritty.yml
        vars:
          font_size: 11
          opacity: 0.8

  linux_sys:
  - { type: cp, src: pkgs/autofs,         dest: /etc/autofs }
  - { type: cp, src: pkgs/fcitx5,         dest: /usr/share/fcitx5 }
  - { type: cp, src: pkgs/iptables,       dest: /etc/iptables }
  - { type: cp, src: pkgs/networkmanager, dest: /etc/NetworkManager }
  - { type: cp, src: pkgs/paru,           dest: /etc/paru }
  - { type: cp, src: pkgs/sshd,           dest: /etc/ssh }
  - { type: cp, src: pkgs/systemd,        dest: /etc/systemd }
  - { type: cp, src: pkgs/udev,           dest: /etc/udev/rules.d }
  - { type: cp, src: pkgs/wallpaper,      dest: /opt/wallpaper }

  wayland:
  - { type: cp, src: pkgs/sway,   dest: "{{env.XDG_CONFIG_HOME}}/sway" }
  - { type: cp, src: pkgs/waybar, dest: "{{env.XDG_CONFIG_HOME}}/waybar" }

  private_unix:
  - { type: cp, src: pkgs/ssh/config,    dest: "{{env.HOME}}/.ssh/config" }
  - { type: cp, src: pkgs/git/gitconfig, dest: "{{env.HOME}}/.gitconfig" }

scenarios:
- name: "sakanainu"
  match:
  - hostname: "^sakanainu$"
  tasks:
  - unix_common
  - rust_unix_common
  - wayland
  - private_unix
- name: "sakanainu-root"
  match:
  - hostname: "^sakanainu$"
  - root: true
  tasks:
  - linux_sys
```

## License

[The Unlicense](https://unlicense.org/)
