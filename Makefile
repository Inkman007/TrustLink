# ─────────────────────────────────────────────────────────────────────────────
# TrustLink Makefile
# ─────────────────────────────────────────────────────────────────────────────
#
# Network targeting
# -----------------
# NETWORK      — target network name (default: testnet)
#                Recognised values: testnet | mainnet | local
#
# The three networks are pre-configured with their canonical RPC URLs and
# network passphrases. You can override any URL via environment variables:
#
#   TESTNET_RPC_URL   (default: https://soroban-testnet.stellar.org)
#   MAINNET_RPC_URL   (default: https://mainnet.stellar.validationcloud.io/v1/...)
#   LOCAL_RPC_URL     (default: http://localhost:8000/soroban/rpc)
#
# Signing identity
# ----------------
# ADMIN_SECRET  — Stellar secret key (S...) used to sign deploy/invoke txns.
#                 Required for deploy and invoke targets.
#                 Never hard-code this value; pass it via the environment:
#                   export ADMIN_SECRET=SXXX...
#                   make deploy
#
# Contract ID
# -----------
# CONTRACT_ID   — Required for invoke target. Set after a successful deploy:
#                   export CONTRACT_ID=C...
#                   make invoke ARGS="-- get_admin"
#
# ─────────────────────────────────────────────────────────────────────────────

NETWORK      ?= testnet
WASM          = target/wasm32-unknown-unknown/release/trustlink.wasm
WASM_OPT      = target/wasm32-unknown-unknown/release/trustlink.optimized.wasm

# ── RPC URLs (overridable via environment) ────────────────────────────────────
TESTNET_RPC_URL  ?= https://soroban-testnet.stellar.org
MAINNET_RPC_URL  ?= https://mainnet.stellar.validationcloud.io/v1/wI7lMGrm7ZU5UP9jKa7R3A
LOCAL_RPC_URL    ?= http://localhost:8000/soroban/rpc

# ── Network passphrases ───────────────────────────────────────────────────────
TESTNET_PASSPHRASE  = Test SDF Network ; September 2015
MAINNET_PASSPHRASE  = Public Global Stellar Network ; September 2015
LOCAL_PASSPHRASE    = Standalone Network ; February 2017

# ── Resolve active network settings ──────────────────────────────────────────
ifeq ($(NETWORK),mainnet)
  RPC_URL    = $(MAINNET_RPC_URL)
  PASSPHRASE = $(MAINNET_PASSPHRASE)
else ifeq ($(NETWORK),local)
  RPC_URL    = $(LOCAL_RPC_URL)
  PASSPHRASE = $(LOCAL_PASSPHRASE)
else
  # Default: testnet
  NETWORK    = testnet
  RPC_URL    = $(TESTNET_RPC_URL)
  PASSPHRASE = $(TESTNET_PASSPHRASE)
endif

.PHONY: build test optimize clean install fmt clippy \
        deploy invoke \
        testnet mainnet local \
        help

# ─────────────────────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────────────────────
help:
	@echo ""
	@echo "TrustLink — Makefile targets"
	@echo "============================"
	@echo ""
	@echo "Build & test"
	@echo "  make build          Build the contract (WASM release)"
	@echo "  make test           Run all unit tests"
	@echo "  make optimize       Build + optimize the WASM artifact"
	@echo "  make fmt            Format source code"
	@echo "  make clippy         Run clippy linter"
	@echo "  make clean          Remove build artifacts"
	@echo "  make install        Print dependency installation instructions"
	@echo ""
	@echo "Deployment  (requires ADMIN_SECRET env var)"
	@echo "  make deploy                     Deploy to testnet (default)"
	@echo "  make deploy NETWORK=testnet     Deploy to testnet"
	@echo "  make deploy NETWORK=mainnet     Deploy to mainnet"
	@echo "  make deploy NETWORK=local       Deploy to local node"
	@echo ""
	@echo "  make testnet                    Alias for deploy NETWORK=testnet"
	@echo "  make mainnet                    Alias for deploy NETWORK=mainnet"
	@echo "  make local                      Alias for deploy NETWORK=local"
	@echo ""
	@echo "Contract invocation  (requires CONTRACT_ID and ADMIN_SECRET env vars)"
	@echo "  make invoke ARGS='-- get_admin'"
	@echo "  make invoke ARGS='-- is_paused'"
	@echo "  make invoke NETWORK=mainnet ARGS='-- get_global_stats'"
	@echo ""
	@echo "Network RPC URLs (override via environment)"
	@echo "  TESTNET_RPC_URL  (default: $(TESTNET_RPC_URL))"
	@echo "  MAINNET_RPC_URL  (default: $(MAINNET_RPC_URL))"
	@echo "  LOCAL_RPC_URL    (default: $(LOCAL_RPC_URL))"
	@echo ""

# ─────────────────────────────────────────────────────────────────────────────
# Build & test
# ─────────────────────────────────────────────────────────────────────────────
install:
	@echo "Required dependencies:"
	@echo "  Rust:        https://rustup.rs/"
	@echo "  Stellar CLI: cargo install --locked stellar-cli --features opt"
	@echo "  WASM target: rustup target add wasm32-unknown-unknown"

build:
	@echo "Building TrustLink ($(NETWORK))..."
	cargo build --target wasm32-unknown-unknown --release

test:
	@echo "Running tests..."
	cargo test

optimize: build
	@echo "Optimizing WASM..."
	stellar contract optimize --wasm $(WASM)
	@echo "Optimized artifact: $(WASM_OPT)"

clean:
	@echo "Cleaning build artifacts..."
	cargo clean

fmt:
	@echo "Formatting code..."
	cargo fmt

clippy:
	@echo "Running clippy..."
	cargo clippy --all-targets -- -D warnings

# ─────────────────────────────────────────────────────────────────────────────
# Deployment
# ─────────────────────────────────────────────────────────────────────────────

# Guard: ADMIN_SECRET must be set for any network operation.
_require_secret:
	@if [ -z "$(ADMIN_SECRET)" ]; then \
	  echo ""; \
	  echo "ERROR: ADMIN_SECRET is not set."; \
	  echo "  export ADMIN_SECRET=SXXX..."; \
	  echo "  make deploy NETWORK=$(NETWORK)"; \
	  echo ""; \
	  exit 1; \
	fi

# Guard: CONTRACT_ID must be set for invoke.
_require_contract:
	@if [ -z "$(CONTRACT_ID)" ]; then \
	  echo ""; \
	  echo "ERROR: CONTRACT_ID is not set."; \
	  echo "  export CONTRACT_ID=C..."; \
	  echo "  make invoke ARGS='-- get_admin'"; \
	  echo ""; \
	  exit 1; \
	fi

# Mainnet requires an explicit confirmation to prevent accidental deploys.
_confirm_mainnet:
	@if [ "$(NETWORK)" = "mainnet" ]; then \
	  echo ""; \
	  echo "WARNING: You are about to deploy to MAINNET."; \
	  printf "Type 'yes' to continue: "; \
	  read confirm; \
	  if [ "$$confirm" != "yes" ]; then \
	    echo "Aborted."; \
	    exit 1; \
	  fi; \
	fi

## deploy — build the optimized WASM and deploy to $(NETWORK).
##
## Usage:
##   make deploy                     # testnet
##   make deploy NETWORK=mainnet     # mainnet (prompts for confirmation)
##   make deploy NETWORK=local       # local node
##
## Outputs the CONTRACT_ID on success.
deploy: _require_secret _confirm_mainnet optimize
	@echo ""
	@echo "Deploying to $(NETWORK) ($(RPC_URL))..."
	@echo ""
	stellar contract deploy \
	  --source "$(ADMIN_SECRET)" \
	  --network-passphrase "$(PASSPHRASE)" \
	  --rpc-url "$(RPC_URL)" \
	  --wasm "$(WASM_OPT)"
	@echo ""
	@echo "Save the contract ID above, then run:"
	@echo "  export CONTRACT_ID=<id>"
	@echo "  make invoke NETWORK=$(NETWORK) ARGS='-- initialize --admin <ADMIN_ADDRESS> --ttl_days null'"

## testnet / mainnet / local — convenience aliases for deploy.
testnet:
	$(MAKE) deploy NETWORK=testnet

mainnet:
	$(MAKE) deploy NETWORK=mainnet

local:
	$(MAKE) deploy NETWORK=local

# ─────────────────────────────────────────────────────────────────────────────
# Contract invocation
# ─────────────────────────────────────────────────────────────────────────────

## invoke — call a function on a deployed contract.
##
## Usage:
##   make invoke ARGS='-- get_admin'
##   make invoke ARGS='-- is_paused'
##   make invoke ARGS='-- initialize --admin G... --ttl_days null'
##   make invoke NETWORK=mainnet ARGS='-- get_global_stats'
##
## Read-only calls do not require ADMIN_SECRET (omit --source).
## State-changing calls require ADMIN_SECRET.
invoke: _require_contract
	@echo "Invoking on $(NETWORK) (contract: $(CONTRACT_ID))..."
	stellar contract invoke \
	  --id "$(CONTRACT_ID)" \
	  --network-passphrase "$(PASSPHRASE)" \
	  --rpc-url "$(RPC_URL)" \
	  $(if $(ADMIN_SECRET),--source "$(ADMIN_SECRET)",) \
	  $(ARGS)
