SYSTEMD_SOURCES   := $(wildcard pkgs/systemd-service/*.service)

SYSTEMD_TARGETS   := $(patsubst pkgs/systemd-service/%,/lib/systemd/system/%,$(SYSTEMD_SOURCES))

.PHONY: install-system

install-system: $(SYSTEMD_TARGETS)
	systemctl daemon-reload

.PHONY: clean-system
clean-system:
	rm -f $(SYSTEMD_TARGETS)

/lib/systemd/system/%: pkgs/systemd-service/%
	bash ../copy.sh $< $@
