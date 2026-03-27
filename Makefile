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
	cp target/$(target)/wallpaper-gui ~/bin/wallpaper-gui
	cp target/$(target)/wallpaper-runner ~/bin/wallpaper-runner

all: build install

uninstall:
	rm target/$(target)/wallpaper-gui
	rm target/$(target)/wallpaper-runner

help:
	@echo "usage: make $(prog) [debug=1]"