SHELL := /usr/bin/env bash

BUN ?= bun

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
ifneq ("$(wildcard /usr/bin/pkg-config)","")
PKG_CONFIG_BIN := /usr/bin/pkg-config
else
PKG_CONFIG_BIN := pkg-config
endif
else
PKG_CONFIG_BIN := pkg-config
endif

.PHONY: help install ui-install desktop-install ui-dev ui-build ui-check ui-e2e \
	desktop-dev desktop-test desktop-build rust-test app-host-test compat-test \
	session-test fmt-check verify-live verify-tauri verify-served verify-gateway \
	verify-execution verify-all

.DEFAULT_GOAL := help

help:
	@echo "MatrixClaw dev shortcuts"
	@echo ""
	@echo "Install:"
	@echo "  make install            Install JS deps with Bun for UI and desktop shell"
	@echo "  make ui-install         Install UI deps"
	@echo "  make desktop-install    Install desktop shell deps"
	@echo ""
	@echo "UI:"
	@echo "  make ui-dev             Run SvelteKit dev server"
	@echo "  make ui-build           Build static UI assets"
	@echo "  make ui-check           Run Svelte checks"
	@echo "  make ui-e2e             Run Playwright e2e suite"
	@echo ""
	@echo "Desktop shell:"
	@echo "  make desktop-dev        Run Tauri dev shell"
	@echo "  make desktop-test       Run desktop shell JS tests"
	@echo "  make desktop-build      Build Tauri release artifacts (uses system pkg-config on Linux)"
	@echo ""
	@echo "Rust:"
	@echo "  make rust-test          Run full Cargo test suite"
	@echo "  make app-host-test      Run app-host tests"
	@echo "  make compat-test        Run compat-openclaw tests"
	@echo "  make session-test       Run session-runtime tests"
	@echo "  make fmt-check          Run rustfmt check"
	@echo ""
	@echo "Verification scripts:"
	@echo "  make verify-live        Run live provider/runtime harness"
	@echo "  make verify-tauri       Run packaged Tauri product verification"
	@echo "  make verify-served      Run served transport verification"
	@echo "  make verify-gateway     Run matrix gateway verification"
	@echo "  make verify-execution   Run execution-node verification"
	@echo "  make verify-all         Run all core verification scripts"

install: ui-install desktop-install

ui-install:
	$(BUN) install --cwd ui

desktop-install:
	$(BUN) install --cwd apps/desktop-shell

ui-dev:
	$(BUN) run --cwd ui dev

ui-build:
	$(BUN) run --cwd ui build

ui-check:
	$(BUN) run --cwd ui check

ui-e2e:
	$(BUN) run --cwd ui test:e2e

desktop-dev:
	PKG_CONFIG="$(PKG_CONFIG_BIN)" $(BUN) run --cwd apps/desktop-shell dev

desktop-test:
	$(BUN) run --cwd apps/desktop-shell test

desktop-build:
	PKG_CONFIG="$(PKG_CONFIG_BIN)" $(BUN) run --cwd apps/desktop-shell build

rust-test:
	cargo test

app-host-test:
	cargo test -p zstar-app-host

compat-test:
	cargo test -p zstar-compat-openclaw

session-test:
	cargo test -p zstar-session-runtime

fmt-check:
	cargo fmt --all --check

verify-live:
	./scripts/verify-live-runtime.sh

verify-tauri:
	./scripts/verify-tauri-product.sh

verify-served:
	./scripts/verify-served-transports.sh

verify-gateway:
	./scripts/verify-matrix-gateway.sh

verify-execution:
	./scripts/verify-execution-node.sh

verify-all: verify-served verify-gateway verify-execution verify-tauri
