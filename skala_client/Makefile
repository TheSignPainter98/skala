LUA = luajit
SOURCES = $(shell find -name '*.yue')
OBJECTS = $(patsubst %.yue,%.lua,$(SOURCES))

all: bin/skala

bin/skala: skala/main.lua scripts/moonpack.lua $(OBJECTS)
	@mkdir -p bin/
	$(LUA) scripts/moonpack.lua $< -o $@

install: scripts/install bin/skala
	./$<

%.lua: %.yue
	yue --target=5.1 -l -s --path="?.yue" $< -o $@
	@touch $@
.PRECIOUS: %.lua

scripts/moonpack.yue: skala/clap.lua

clean:
	$(RM) bin/skala $(OBJECTS)
.PHONY: clean
