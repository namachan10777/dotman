SERVICE_SOURCES   := $(wildcard services/*.service)
VIM_SOURCE        := pkgs/vim/init.vim
FISH_SOURCE       := pkgs/fish/conf.d/init.fish
SYSTEMD_SOURCES   := $(shell find pkgs/systemd -type f)

SERVICE_TARGETS   := $(patsubst services/%,/lib/systemd/system/%,$(SERVICE_SOURCES))
VIM_TARGET        := $(XDG_CONFIG_HOME)/nvim/init.vim $(HOME)/.vimrc
FISH_TARGET       := $(XDG_CONFIG_HOME)/fish/conf.d/init.fish
SYSTEMD_TARGETS   := $(patsubst pkgs/systemd/%,/etc/systemd/%,$(SYSTEMD_SOURCES))

.PHONY: install
install: $(VIM_TARGET) $(FISH_TARGET)

.PHONY: clean
clean:
	rm -f $(VIM_TARGET)
	rm -f $(FISH_TARGET)

.PHONY: install-system
install-system: $(SERVICE_TARGETS) $(SYSTEMD_TARGETS)
	systemctl daemon-reload

.PHONY: clean-system
clean-system:
	rm -f $(SERVICE_TARGETS)

/lib/systemd/system/%: services/%
	bash ../copy.sh $< $@

$(SYSTEMD_TARGETS): $(SYSTEMD_SOURCES)
	bash ../copy.sh $< $@

$(FISH_TARGET): $(FISH_SOURCE)
	bash ../copy.sh $< $@

$(VIM_TARGET): $(VIM_SOURCE)
	bash ../copy.sh $< $@
