YUE_SOURCES = $(shell find -name '*.yue')
RUST_SOURCES = $(shell find -name '*.rs')

install: install-skala_client # install-skala_server
.PHONY: install

install-skala_client: skala_client/Makefile skala_client/bin/skala $(YUE_SOURCES)
	make -C skala_client install
.PHONY: install
