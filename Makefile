# Makefile for RemitSave Africa

.PHONY: all build test clean build-contracts build-backend bootstrap

all: build

bootstrap:
	bash scripts/bootstrap.sh

build: build-contracts build-backend

build-contracts:
	cd contracts && cargo build --target wasm32-unknown-unknown --release

build-backend:
	cd backend && cargo build --release

test: test-contracts test-backend

test-contracts:
	cd contracts && cargo test

test-backend:
	cd backend && cargo test

clean:
	cd contracts && cargo clean
	cd backend && cargo clean

# Helper to build specific contract
remit-save:
	cd contracts/remit-save && cargo build --target wasm32-unknown-unknown --release

vault-pool:
	cd contracts/vault-pool && cargo build --target wasm32-unknown-unknown --release
