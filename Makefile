UNAME_S := $(shell uname -s)
BINARY_NAME := daneel-web

ifeq ($(UNAME_S),Linux)
    TARGET := x86_64-unknown-linux-musl
    CARGO_FLAGS := --release --target $(TARGET)
    BINARY_PATH := target/$(TARGET)/release/$(BINARY_NAME)
else
    CARGO_FLAGS := --release
    BINARY_PATH := target/release/$(BINARY_NAME)
endif

.PHONY: all build install clean

all: build install

build:
	cargo build $(CARGO_FLAGS)

install: build
	mkdir -p $(HOME)/bin
	cp $(BINARY_PATH) $(HOME)/bin/$(BINARY_NAME)

clean:
	cargo clean
