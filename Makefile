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


# TEST

lint: lint-commit-messages
lint-commit-messages:
	./scripts/lint-commit-messages.sh


# TOOLBOX

toolbox-build:
	cd toolbox && \
	$(CONTAINER_MGR) build --no-cache -t localhost/waketimed_toolbox_builder:latest . && \
	$(CONTAINER_MGR) tag localhost/waketimed_toolbox_builder:latest localhost/waketimed_toolbox_builder:$$(date "+%Y_%m_%d")

toolbox-clean:
	$(CONTAINER_MGR) rmi localhost/waketimed_toolbox_builder:latest || true
