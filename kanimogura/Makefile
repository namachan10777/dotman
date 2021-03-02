HOST_SOURCE = hosts hostname
HOST_TARGET = /etc/hosts /etc/hostname

.PHONY: install
install: $(HOST_TARGET)

.PHONY: clean
clean:
	rm -f $(HOST_TARGET)

$(HOST_TARGET): $(HOST_SOURCE)
	bash ../copy.sh $< $@
