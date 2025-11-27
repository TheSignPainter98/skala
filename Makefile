LUA = luajit
SOURCES = $(shell find -name '*.yue')
OBJECTS = $(patsubst %.yue,%.lua,$(SOURCES))

all: bin/skala

bin/skala: skala/main.lua moonpack.lua $(OBJECTS)
	@mkdir -p bin/
	$(LUA) moonpack.lua $< -o $@

install: skala

%.lua: %.yue
	yue --target=5.1 -l -s --path="?.yue" $< -o $@
	@touch $@
.PRECIOUS: %.lua

moonpack.yue: skala/clap.lua

clean:
	$(RM) bin/skala $(OBJECTS) moonpack.lua
.PHONY: clean
