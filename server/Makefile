SYSTEMD_SOURCES   := $(wildcard pkgs/systemd-service/*.service)
VIM_SOURCE        := pkgs/vim/init.vim
FISH_SOURCE       := pkgs/fish/conf.d/init.fish

SYSTEMD_TARGETS   := $(patsubst pkgs/systemd-service/%,/lib/systemd/system/%,$(SYSTEMD_SOURCES))
VIM_TARGET        := $(XDG_CONFIG_HOME)/nvim/init.vim $(HOME)/.vimrc
FISH_TARGET       := $(XDG_CONFIG_HOME)/fish/conf.d/init.fish

.PHONY: install
install: $(VIM_TARGET) $(FISH_TARGET)

.PHONY: clean
clean:
	rm -f $(VIM_TARGET)
	rm -f $(FISH_TARGET)

.PHONY: install-system
install-system: $(SYSTEMD_TARGETS)
	systemctl daemon-reload

.PHONY: clean-system
clean-system:
	rm -f $(SYSTEMD_TARGETS)

/lib/systemd/system/%: pkgs/systemd-service/%
	bash ../copy.sh $< $@

$(FISH_TARGET): $(FISH_SOURCE)
	bash ../copy.sh $< $@

$(VIM_TARGET): $(VIM_SOURCE)
	bash ../copy.sh $< $@
