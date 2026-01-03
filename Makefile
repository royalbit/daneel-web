BINARY_NAME := daneel-web

# Native builds (glibc) - ONNX Runtime doesn't support musl
CARGO_FLAGS := --release
BINARY_PATH := target/release/$(BINARY_NAME)

.PHONY: all build install clean

all: build install

build:
	cargo build $(CARGO_FLAGS)

install: build
	mkdir -p $(HOME)/bin
	cp $(BINARY_PATH) $(HOME)/bin/$(BINARY_NAME)

clean:
	cargo clean
