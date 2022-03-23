.DEFAULT_GOAL := build-release
SHELL := /bin/bash
ROOT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

CONTAINER_MGR ?= podman
WAKETIMED_BUILD_PROFILE ?= --release


# BUILD

build:
	cargo build $(WAKETIMED_BUILD_PROFILE)

build-debug: WAKETIMED_BUILD_PROFILE =
build-debug: build

build-release: WAKETIMED_BUILD_PROFILE = --release
build-release: build

clean:
	cargo clean

deps-check-outdated:
	if ! cargo outdated -h &> /dev/null; then \
		cargo install cargo-outdated; \
	fi; \
	cargo outdated

deps-update:
	cargo update

doc:
	cargo doc


# LINT

lint: lint-commit-messages lint-clippy lint-fmt

lint-commit-messages:
	./scripts/lint-commit-messages.sh

lint-fmt:
	cargo fmt --check

lint-clippy:
	cargo clippy -- -D warnings $(WAKETIMED_LINT_CLIPPY_ARGS)


fix: fix-clippy fix-fmt

fix-clippy:
	cargo clippy --fix --allow-dirty --allow-staged

fix-fmt:
	cargo fmt


# TEST

test: test-waketimed_core test-waketimed test-waketimectl

test-waketimed:
	cd waketimed && cargo test

test-waketimed_core:
	cd waketimed_core && cargo test

test-waketimectl:
	cd waketimectl && cargo test

# TOOLBOX

toolbox-build:
	cd toolbox && \
	$(CONTAINER_MGR) build --no-cache -t localhost/waketimed_toolbox_builder:latest . && \
	$(CONTAINER_MGR) tag localhost/waketimed_toolbox_builder:latest localhost/waketimed_toolbox_builder:$$(date "+%Y_%m_%d")

toolbox-clean:
	$(CONTAINER_MGR) rmi localhost/waketimed_toolbox_builder:latest || true
