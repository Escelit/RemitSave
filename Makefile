# Makefile for RemitSave Africa

.PHONY: all build test clean build-contracts build-backend

all: build

build: build-contracts build-backend

build-contracts:
	cd contracts && cargo build --target wasm32-unknown-unknown --release

build-backend:
	cd backend && cargo build --release

test:
	cd contracts && cargo test
	# cd backend && cargo test

clean:
	cd contracts && cargo clean
	cd backend && cargo clean

# Helper to build specific contract
remit-save:
	cd contracts/remit-save && cargo build --target wasm32-unknown-unknown --release

vault-pool:
	cd contracts/vault-pool && cargo build --target wasm32-unknown-unknown --release
