LUA = luajit
SOURCES = $(shell find -name '*.yue')
OBJECTS = $(patsubst %.yue,%.lua,$(SOURCES))

all: skala

skala: src/main.lua
	cp $< $@

install: skala

%.lua: %.yue
	yue --target=5.1 -l -s --path="?.yue" $< -o $@
	@touch $@
.PRECIOUS: %.lua

clean:
	$(RM) skala $(OBJECTS) moonpack.lua
.PHONY: clean
