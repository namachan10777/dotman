ALACRITTY_SOURCES := pkgs/alacritty/alacritty.yml
FISH_SOURCES      := $(shell find pkgs/fish -type f)
GIT_SOURCE        := pkgs/git/gitconfig
LATEXMK_SOURCE    := pkgs/latexmk/latexmkrc
LAZYGIT_SOURCE    := pkgs/lazygit/config.yml
NVIM_SOURCES      := $(shell find pkgs/neovim -type f)
SWAY_SOURCE       := pkgs/sway/config
TIG_SOURCE        := pkgs/tig/tigrc

UDEV_SOURCES      := $(shell find pkgs/udev -name *.rules -type f)
IPTABLES_SOURCES  := $(shell find pkgs/iptables/ -name *.rules -type f)

ALACRITTY_TARGETS := $(XDG_CONFIG_HOME)/alacritty/alacritty.yml
FISH_TARGETS      := $(patsubst pkgs/fish/%,$(XDG_CONFIG_HOME)/fish/%,$(FISH_SOURCES))
GIT_TARGET        := $(HOME)/.gitconfig
LATEXMK_TARGET    := $(HOME)/.latexmkrc
LAZYGIT_TARGET    := $(XDG_CONFIG_HOME)/jessedufield/config.yml
NVIM_TARGETS      := $(patsubst pkgs/neovim/%,$(XDG_CONFIG_HOME)/nvim/%,$(NVIM_SOURCES))
SWAY_TARGET       := $(XDG_CONFIG_HOME)/sway/config
TIG_TARGET        := $(HOME)/.tigrc

UDEV_TARGETS      := $(patsubst pkgs/udev/%,/etc/udev/rules.d/%,$(UDEV_SOURCES))
IPTABLES_TARGETS  := $(patsubst pkgs/iptables/%,/etc/iptables/%,$(IPTABLES_SOURCES))

UTIL_SOURCES     := $(wildcard bin/*)
UTIL_TARGETS     := $(patsubst bin/%,/usr/local/bin/%,$(UTIL_SOURCES))

.PHONY: install
install: \
	$(FISH_TARGETS) \
	$(ALACRITTY_TARGETS) \
	$(GIT_TARGET) \
	$(LATEXMK_TARGET) \
	$(LAZYGIT_TARGET) \
	$(NVIM_TARGETS) \
	$(SWAY_TARGET) \
	$(TIG_TARGET)

.PHONY: install-system
install-system: $(UDEV_TARGETS) $(IPTABLES_TARGETS) $(UTIL_TARGETS)

.PHONY: clean
clean:
	@echo $(FISH_TARGETS)
	@echo $(FISH_SOURCES)
	rm -f $(ALACRITTY_TARGETS)
	rm -rf $(FISH_TARGETS)
	rm -f $(LATEXMK_TARGET)
	rm -f $(LAZYGIT_TARGET)
	rm -rf $(NVIM_TARGETS)
	rm -f $(SWAY_TARGET)
	rm -f $(TIG_TARGET)

.PHONY: clean-system
clean-system:
	@echo $(IPTABLES_SOURCES)
	@echo $(IPTABLES_TARGETS)
	rm -f $(UDEV_TARGETS) $(IPTABLES_TARGETS)
	rm -f $(UTIL_TARGETS)

$(GIT_TARGET): $(GIT_SOURCE)
	bash copy.sh $< $@

$(LATEXMK_TARGET): $(LATEXMK_SOURCE)
	bash copy.sh $< $@

$(TIG_TARGET): $(TIG_SOURCE)
	bash copy.sh $< $@

$(SWAY_TARGET): $(SWAY_SOURCE)
	bash copy.sh $< $@

$(LAZYGIT_TARGET): $(LAZYGIT_SOURCE)
	bash copy.sh $< $@

$(XDG_CONFIG_HOME)/nvim/%: pkgs/neovim/%
	bash copy.sh $< $@

$(XDG_CONFIG_HOME)/alacritty/alacritty.yml: pkgs/alacritty/alacritty.yml hooks/set_alacritty_font_size.sh
	bash copy.sh $< $@
	bash hooks/set_alacritty_font_size.sh $@

$(XDG_CONFIG_HOME)/fish/%: pkgs/fish/%
	bash copy.sh $< $@

/etc/udev/rules.d/%: pkgs/udev/%
	bash copy.sh $< $@

/etc/iptables/%: pkgs/iptables/%
	bash copy.sh $< $@

/usr/local/bin/%: bin/%
	bash copy.sh $< $@
