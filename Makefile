prog :=xnixperms

debug ?=

$(info debug is $(debug))

ifdef debug
  release :=
  target :=debug
  extension :=debug
else
  release :=--release
  target :=release
  extension :=
endif

build:
	cargo build $(release)

install:
	cp target/$(target)/wallpaper-ctl ~/bin/wallpaper-ctl
	cp target/$(target)/wallpaper-manager ~/bin/wallpaper-manager
	cp target/$(target)/wallpaper-engine ~/bin/wallpaper-engine

all: build install

uninstall:
	rm target/$(target)/wallpaper-ctl
	rm target/$(target)/wallpaper-gui
	rm target/$(target)/wallpaper-engine

help:
	@echo "usage: make $(prog) [debug=1]"