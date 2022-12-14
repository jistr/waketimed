.DEFAULT_GOAL := build-release
SHELL := /bin/bash
ROOT_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))

CONTAINER_MGR ?= podman
KEEP_TOOLBOX_IMAGES ?= 2

WAKETIMED_BUILD_PROFILE ?= --release
WAKETIMED_INSTALL_PROFILE ?= release
WAKETIMED_INSTALL_BIN_DIR ?= /usr/local/bin
WAKETIMED_INSTALL_BIN_NAME ?= waketimed
WAKETIMED_INSTALL_SERVICE_DIR ?= /etc/systemd/system
WAKETIMED_INSTALL_SERVICE_NAME ?= waketimed.service
WAKETIMED_TEST_INT_ARGS ?= -- --nocapture
export WAKETIMED_BUS_ADDRESS ?= $(DBUS_SESSION_BUS_ADDRESS)

# BUILD

build:
	cargo build $(WAKETIMED_BUILD_PROFILE)

build-debug: WAKETIMED_BUILD_PROFILE =
build-debug: build

build-release: WAKETIMED_BUILD_PROFILE = --release
build-release: build

build-release-aarch64: export PKG_CONFIG_ALLOW_CROSS = 1
build-release-aarch64: WAKETIMED_BUILD_PROFILE = --release --target aarch64-unknown-linux-gnu
build-release-aarch64: cross-prep-cargo build

install:
	install -m 0755 target/$(WAKETIMED_INSTALL_PROFILE)/waketimed $(WAKETIMED_INSTALL_BIN_DIR)/$(WAKETIMED_INSTALL_BIN_NAME)

install-service:
	install -m 0644 waketimed/config/systemd/waketimed.service $(WAKETIMED_INSTALL_SERVICE_DIR)/$(WAKETIMED_INSTALL_SERVICE_NAME)

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

publish: lint test
	cargo publish -p waketimed

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

test: test-unit test-int

test-unit: test-unit-waketimed

test-int: test-int-waketimed

test-unit-waketimed:
	cd waketimed && cargo test --bins

test-int-waketimed:
	cd waketimed && RUST_BACKTRACE=1 cargo test --test '*' $(WAKETIMED_TEST_INT_ARGS)


# TOOLBOX

toolbox-build:
	cd toolbox && \
	$(CONTAINER_MGR) build --no-cache -t localhost/waketimed_toolbox_builder:latest . && \
	$(CONTAINER_MGR) tag localhost/waketimed_toolbox_builder:latest localhost/waketimed_toolbox_builder:$$(date "+%Y_%m_%d")

toolbox-clean: KEEP_TOOLBOX_IMAGES=0
toolbox-clean: toolbox-expire
	$(CONTAINER_MGR) rmi localhost/waketimed_toolbox_builder:latest || true

toolbox-expire:
	TAGS=$$(podman images | grep '^localhost/waketimed_toolbox_builder ' | grep -E ' [0-9]{4}_[0-9]{2}_[0-9]{2} ' | awk '{ print $$2;}' | sort -r); \
	IDX=0 ; \
	for TAG in $$TAGS; do \
		if [ "$$IDX" -ge "$(KEEP_TOOLBOX_IMAGES)" ]; then \
			podman rmi localhost/waketimed_toolbox_builder:$$TAG; \
		fi; \
		: $$(( IDX++ )); \
	done

cross-toolbox-build:
	cd toolbox && \
	$(CONTAINER_MGR) build --no-cache -f Containerfile_cross -t localhost/waketimed_toolbox_cross:latest . && \
	$(CONTAINER_MGR) tag localhost/waketimed_toolbox_cross:latest localhost/waketimed_toolbox_cross:$$(date "+%Y_%m_%d")

cross-toolbox-clean: KEEP_TOOLBOX_IMAGES=0
cross-toolbox-clean: cross-toolbox-expire
	$(CONTAINER_MGR) rmi localhost/waketimed_toolbox_cross:latest || true

cross-toolbox-expire:
	TAGS=$$(podman images | grep '^localhost/waketimed_toolbox_cross ' | grep -E ' [0-9]{4}_[0-9]{2}_[0-9]{2} ' | awk '{ print $$2;}' | sort -r); \
	IDX=0 ; \
	for TAG in $$TAGS; do \
		if [ "$$IDX" -ge "$(KEEP_TOOLBOX_IMAGES)" ]; then \
			podman rmi localhost/waketimed_toolbox_cross:$$TAG; \
		fi; \
		: $$(( IDX++ )); \
	done

cross-prep-cargo:
	mkdir -p tmp/cargo-cross; \
	if ! grep /usr/bin/aarch64-linux-gnu-gcc tmp/cargo-cross/config &> /dev/null; then \
		echo '[target.aarch64-unknown-linux-gnu]' >>tmp/cargo-cross/config; \
		echo 'linker = "/usr/bin/aarch64-linux-gnu-gcc"' >>tmp/cargo-cross/config; \
	fi
