SHELL       = /bin/bash

RUST_FILES  = $(shell find . -type f -name '*.rs')
CARGO_PROJ  = kernel
CARGO_TOML  = $(CARGO_PROJ)/Cargo.toml
CARGO_FLAGS = --verbose --color=always

.PHONY: all
all: build

.PHONY: init
init:
	bin/init.sh

.PHONY: build
build: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) build

.PHONY: release
release: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) build --release

.PHONY: test
test: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) test

.PHONY: test-debug
test-debug: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && HALOGEN_DEBUG=1 cargo $(CARGO_FLAGS) test

.PHONY: run
run: build
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) run

.PHONY: run-debug
run-debug: $(RUST_FILES) $(CARGO_TOML)
	cd $(CARGO_PROJ) && HALOGEN_DEBUG=1 cargo $(CARGO_FLAGS) run

.PHONY: attach
attach:
	bin/attach.sh

.PHONY: clean
clean:
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) clean

.PHONY: fmt
fmt:
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) fmt

.PHONY: fmt-check
fmt-check:
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) fmt --check

.PHONY: doc
doc:
	cd $(CARGO_PROJ) && cargo $(CARGO_FLAGS) doc
