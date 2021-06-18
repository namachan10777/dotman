ALACRITTY_SOURCES := pkgs/alacritty/alacritty.yml
PARU_SOURCES      := pkgs/paru/paru.conf
FISH_SOURCES      := $(shell find pkgs/fish -type f)
GIT_SOURCE        := pkgs/git/gitconfig
LATEXMK_SOURCE    := pkgs/latexmk/latexmkrc
LAZYGIT_SOURCE    := pkgs/lazygit/config.yml
NPM_SOURCE        := pkgs/npm/npmrc
NVIM_SOURCES      := $(shell find pkgs/neovim -type f)
FCITX5_SOURCES    := $(shell find pkgs/fcitx5 -type f)
SWAY_SOURCE       := pkgs/sway/config
TIG_SOURCE        := pkgs/tig/tigrc
GPG_SOURCES       := $(wildcard pkgs/gpg/*)
SSH_SOURCE        := pkgs/ssh/config
VSCODE_SOURCES    := $(wildcard pkgs/vscode/*)

UDEV_SOURCES      := $(shell find pkgs/udev -name *.rules -type f)
IPTABLES_SOURCES  := $(shell find pkgs/iptables/ -name *.rules -type f)
SYSTEMD_SOURCES   := $(shell find pkgs/systemd/ -type f)
SSHD_SOURCE       := pkgs/sshd/sshd_config
AUTOFS_SOURCE     := $(shell find pkgs/autofs/ -type f)
NETWORKMANAGER_SOURCES := $(shell find pkgs/systemd/ -type f)

ALACRITTY_TARGETS := $(XDG_CONFIG_HOME)/alacritty/alacritty.yml
PARU_TARGETS      := $(XDG_CONFIG_HOME)/paru/paru.conf
FISH_TARGETS      := $(patsubst pkgs/fish/%,$(XDG_CONFIG_HOME)/fish/%,$(FISH_SOURCES)) $(XDG_CONFIG_HOME)/fish/completions/kubectl.fish
GIT_TARGET        := $(HOME)/.gitconfig
LATEXMK_TARGET    := $(HOME)/.latexmkrc
LAZYGIT_TARGET    := $(XDG_CONFIG_HOME)/jessedufield/config.yml
NPM_TARGET        := $(HOME)/.npmrc
NVIM_TARGETS      := $(patsubst pkgs/neovim/%,$(XDG_CONFIG_HOME)/nvim/%,$(NVIM_SOURCES))
FCITX5_TARGETS    := $(patsubst pkgs/fcitx5/%,/usr/share/fcitx5/%,$(FCITX5_SOURCES))
SWAY_TARGET       := $(XDG_CONFIG_HOME)/sway/config
TIG_TARGET        := $(HOME)/.tigrc
GPG_TARGETS       := $(patsubst pkgs/gpg/%,$(HOME)/.gnupg/%,$(GPG_SOURCES))
SSH_TARGET        := $(HOME)/.ssh/config
VSCODE_TARGETS    := \
	$(patsubst pkgs/vscode/%,$(XDG_CONFIG_HOME)/Code\ -\ OSS/User/%,$(VSCODE_SOURCES)) \
	$(patsubst pkgs/vscode/%,$(XDG_CONFIG_HOME)/Code/User/%,$(VSCODE_SOURCES))

UDEV_TARGETS      := $(patsubst pkgs/udev/%,/etc/udev/rules.d/%,$(UDEV_SOURCES))
IPTABLES_TARGETS  := $(patsubst pkgs/iptables/%,/etc/iptables/%,$(IPTABLES_SOURCES))
SYSTEMD_TARGETS   := $(patsubst pkgs/systemd/%,/etc/systemd/%,$(SYSTEMD_SOURCES))
SSHD_TARGET       := /etc/ssh/sshd_config
AUTOFS_TARGET     := $(patsubst pkgs/autofs/%,/etc/autofs/%,$(AUTOFS_SOURCE))
NETWORKMANAGER_TARGETS := $(patsubst pkgs/systemd/%,/etc/systemd/%,$(NETWORKMANAGER_SOURCES))

UTIL_SOURCES     := $(wildcard bin/*)
UTIL_TARGETS     := $(patsubst bin/%,/usr/local/bin/%,$(UTIL_SOURCES))

.PHONY: install
install: \
	$(FISH_TARGETS) \
	$(ALACRITTY_TARGETS) \
	$(GIT_TARGET) \
	$(LATEXMK_TARGET) \
	$(LAZYGIT_TARGET) \
	$(NPM_TARGET) \
	$(NVIM_TARGETS) \
	$(SWAY_TARGET) \
	$(TIG_TARGET) \
	$(GPG_TARGETS) \
	$(SSH_TARGET) \
	$(PARU_TARGETS) \
	$(VSCODE_TARGETS)

.PHONY: install-system

install-system: \
	$(UDEV_TARGETS) \
	$(IPTABLES_TARGETS) \
	$(UTIL_TARGETS) \
	$(FCITX5_TARGETS) \
	$(SYSTEMD_TARGETS) \
	$(SSHD_TARGET) \
	$(NETWORKMANAGER_TARGETS) \
	$(AUTOFS_TARGET)

.PHONY: clean
clean:
	rm -f $(ALACRITTY_TARGETS)
	rm -f $(PARU_TARGETS)
	rm -rf $(FISH_TARGETS)
	rm -f $(LATEXMK_TARGET)
	rm -f $(LAZYGIT_TARGET)
	rm -rf $(NVIM_TARGETS)
	rm -f $(SWAY_TARGET)
	rm -f $(TIG_TARGET)
	rm -f $(VSCODE_TARGETS)
	rm -f $(PARU_TARGETS)

.PHONY: clean-system
clean-system:
	@echo $(IPTABLES_SOURCES)
	@echo $(IPTABLES_TARGETS)
	rm -f $(UDEV_TARGETS) $(IPTABLES_TARGETS)
	rm -f $(UTIL_TARGETS)

$(GIT_TARGET): $(GIT_SOURCE)
	bash ../copy.sh $< $@

$(LATEXMK_TARGET): $(LATEXMK_SOURCE)
	bash ../copy.sh $< $@

$(TIG_TARGET): $(TIG_SOURCE)
	bash ../copy.sh $< $@

$(SWAY_TARGET): $(SWAY_SOURCE)
	bash ../copy.sh $< $@

$(LAZYGIT_TARGET): $(LAZYGIT_SOURCE)
	bash ../copy.sh $< $@

$(NPM_TARGET): $(NPM_SOURCE)
	bash ../copy.sh $< $@

.PHONY: nvim
nvim: $(NVIM_TARGETS)
$(XDG_CONFIG_HOME)/nvim/%: pkgs/neovim/%
	bash ../copy.sh $< $@

.PHONY: alacritty
alacritty: $(ALACRITTY_TARGETS)
$(XDG_CONFIG_HOME)/alacritty/alacritty.yml: pkgs/alacritty/alacritty.yml hooks/set_alacritty_font_size.sh
	bash ../copy.sh $< $@
	bash hooks/set_alacritty_font_size.sh $@

$(XDG_CONFIG_HOME)/paru/paru.conf: pkgs/paru/paru.conf
	bash ../copy.sh $< $@

$(XDG_CONFIG_HOME)/fish/completions/kubectl.fish: ../fish-kubectl-completions/completions/kubectl.fish
	bash ../copy.sh $< $@

$(XDG_CONFIG_HOME)/fish/%: pkgs/fish/%
	bash ../copy.sh $< $@

$(HOME)/.gnupg/%: pkgs/gpg/%
	bash ../copy.sh $< $@

/etc/udev/rules.d/%: pkgs/udev/%
	bash ../copy.sh $< $@

/etc/iptables/%: pkgs/iptables/%
	bash ../copy.sh $< $@

/etc/systemd/%: pkgs/systemd/%
	bash ../copy.sh $< $@

$(SSH_TARGET): $(SSH_SOURCE)
	bash ../copy.sh $< $@

$(SSHD_TARGET): $(SSHD_SOURCE)
	bash ../copy.sh $< $@

$(XDG_CONFIG_HOME)/Code\ -\ OSS/User/%: pkgs/vscode/%
	bash ../copy.sh $< "$@"

/etc/NetworkManager/%: pkgs/networkmanager/%
	bash ../copy.sh $< $@

/etc/autofs/%: pkgs/autofs/%
	bash ../copy.sh $< $@

/usr/share/fcitx5/%: pkgs/fcitx5/%
	bash ../copy.sh $< $@

/usr/local/bin/%: bin/%
	bash ../copy.sh $< $@
