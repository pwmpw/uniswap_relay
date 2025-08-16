# Uniswap Relay DApp Makefile
# A robust build system for development, testing, and deployment

# Configuration
PROJECT_NAME := uniswap_relay_dapp
BINARY_NAME := uniswap_relay_dapp
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
BUILD_TIME := $(shell date -u '+%Y-%m-%d_%H:%M:%S_UTC')
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

# Directories
SRC_DIR := src
TEST_DIR := tests
CONFIG_DIR := config
DOCKER_DIR := docker
TARGET_DIR := target
COVERAGE_DIR := coverage
SCRIPTS_DIR := scripts

# Build targets
TARGETS := x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin x86_64-pc-windows-msvc

# Rust toolchain
RUST_VERSION := 1.75
CARGO := cargo
RUSTC := rustc
RUSTUP := rustup

# Docker
DOCKER := docker
DOCKER_COMPOSE := docker-compose
DOCKER_IMAGE := $(PROJECT_NAME)
DOCKER_TAG := $(VERSION)

# Environment
ENV ?= development
ENVIRONMENT ?= development
CONFIG_FILE := $(CONFIG_DIR)/config.toml

# Flags
CARGO_FLAGS :=
CARGO_TEST_FLAGS := --all-features --verbose
CARGO_BUILD_FLAGS := --release
CARGO_CLIPPY_FLAGS := --all-targets --all-features -- -D warnings
CARGO_FMT_FLAGS := --all -- --check

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
WHITE := \033[1;37m
NC := \033[0m # No Color

# Help target
.PHONY: help
help: ## Show this help message
	@echo "$(CYAN)Uniswap Relay DApp - Available Targets$(NC)"
	@echo "$(CYAN)=====================================$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(GREEN)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)Environment Variables:$(NC)"
	@echo "  ENV=development|production  Configuration environment (default: development)"
	@echo "  VERSION=tag                 Override version (default: git tag)"
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make build ENV=production"
	@echo "  make test VERSION=v1.0.0"
	@echo "  make docker-build DOCKER_TAG=latest"

# Development targets
.PHONY: dev
dev: ## Run in development mode
	@echo "$(BLUE)🚀 Starting development mode...$(NC)"
	@echo "$(CYAN)Environment: $(ENVIRONMENT)$(NC)"
	@echo "$(CYAN)Config: $(CONFIG_FILE)$(NC)"
	ENVIRONMENT=$(ENVIRONMENT) $(CARGO) run --bin $(BINARY_NAME)

.PHONY: run
run: ## Run the built binary
	@echo "$(BLUE)🚀 Running binary...$(NC)"
	@echo "$(CYAN)Environment: $(ENVIRONMENT)$(NC)"
	@echo "$(CYAN)Config: $(CONFIG_FILE)$(NC)"
	@if [ ! -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ]; then \
		echo "$(YELLOW)Binary not found. Building first...$(NC)"; \
		$(MAKE) build; \
	fi
	ENVIRONMENT=$(ENVIRONMENT) ./$(TARGET_DIR)/release/$(BINARY_NAME)

.PHONY: dev-watch
dev-watch: ## Run with cargo watch for auto-reload
	@echo "$(BLUE)👀 Starting development mode with auto-reload...$(NC)"
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		cargo install cargo-watch; \
	fi
	cargo watch -x "run --bin $(BINARY_NAME)"

.PHONY: check
check: ## Check code without building
	@echo "$(BLUE)🔍 Checking code...$(NC)"
	$(CARGO) check --all-features

.PHONY: check-all
check-all: ## Check code for all targets
	@echo "$(BLUE)🔍 Checking code for all targets...$(NC)"
	@for target in $(TARGETS); do \
		echo "$(CYAN)Checking $$target...$(NC)"; \
		$(CARGO) check --target $$target --all-features || exit 1; \
	done

# Building targets
.PHONY: build
build: ## Build release binary
	@echo "$(BLUE)🔨 Building release binary...$(NC)"
	@echo "$(CYAN)Version: $(VERSION)$(NC)"
	@echo "$(CYAN)Commit: $(GIT_COMMIT)$(NC)"
	@echo "$(CYAN)Branch: $(GIT_BRANCH)$(NC)"
	@echo "$(CYAN)Build time: $(BUILD_TIME)$(NC)"
	$(CARGO) build $(CARGO_BUILD_FLAGS) $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Build completed!$(NC)"
	@echo "$(CYAN)Binary location: $(TARGET_DIR)/release/$(BINARY_NAME)$(NC)"
	@ls -lh $(TARGET_DIR)/release/$(BINARY_NAME)

.PHONY: build-debug
build-debug: ## Build debug binary
	@echo "$(BLUE)🔨 Building debug binary...$(NC)"
	$(CARGO) build $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Debug build completed!$(NC)"

.PHONY: build-all
build-all: ## Build for all supported targets
	@echo "$(BLUE)🔨 Building for all targets...$(NC)"
	@for target in $(TARGETS); do \
		echo "$(CYAN)Building for $$target...$(NC)"; \
		$(CARGO) build --target $$target $(CARGO_BUILD_FLAGS) $(CARGO_FLAGS) || exit 1; \
	done
	@echo "$(GREEN)✅ All targets built successfully!$(NC)"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(BLUE)🧹 Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	@rm -rf $(COVERAGE_DIR)
	@echo "$(GREEN)✅ Clean completed!$(NC)"

.PHONY: clean-all
clean-all: ## Clean all artifacts including dependencies
	@echo "$(BLUE)🧹 Cleaning all artifacts...$(NC)"
	$(CARGO) clean
	@rm -rf $(COVERAGE_DIR)
	@rm -rf target/
	@rm -rf Cargo.lock
	@echo "$(GREEN)✅ Complete clean finished!$(NC)"

# Testing targets
.PHONY: test
test: ## Run all tests
	@echo "$(BLUE)🧪 Running tests...$(NC)"
	$(CARGO) test $(CARGO_TEST_FLAGS) $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Tests completed!$(NC)"

.PHONY: test-unit
test-unit: ## Run unit tests only
	@echo "$(BLUE)🧪 Running unit tests...$(NC)"
	$(CARGO) test --lib $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Unit tests completed!$(NC)"

.PHONY: test-integration
test-integration: ## Run integration tests only
	@echo "$(BLUE)🧪 Running integration tests...$(NC)"
	$(CARGO) test --test integration $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Integration tests completed!$(NC)"

.PHONY: test-watch
test-watch: ## Run tests with auto-reload
	@echo "$(BLUE)👀 Running tests with auto-reload...$(NC)"
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-watch...$(NC)"; \
		cargo install cargo-watch; \
	fi
	cargo watch -x "test $(CARGO_TEST_FLAGS)"

.PHONY: test-coverage
test-coverage: ## Run tests with coverage report
	@echo "$(BLUE)📊 Running tests with coverage...$(NC)"
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-tarpaulin...$(NC)"; \
		cargo install cargo-tarpaulin; \
	fi
	@mkdir -p $(COVERAGE_DIR)
	cargo tarpaulin --out Html --output-dir $(COVERAGE_DIR)
	@echo "$(GREEN)✅ Coverage report generated in $(COVERAGE_DIR)/$(NC)"

# Code quality targets
.PHONY: fmt
fmt: ## Format code
	@echo "$(BLUE)🎨 Formatting code...$(NC)"
	$(CARGO) fmt --all
	@echo "$(GREEN)✅ Code formatting completed!$(NC)"

.PHONY: fmt-check
fmt-check: ## Check code formatting
	@echo "$(BLUE)🎨 Checking code formatting...$(NC)"
	$(CARGO) fmt $(CARGO_FMT_FLAGS) $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Code formatting check passed!$(NC)"

.PHONY: clippy
clippy: ## Run clippy linter
	@echo "$(BLUE)🔍 Running clippy...$(NC)"
	$(CARGO) clippy $(CARGO_CLIPPY_FLAGS) $(CARGO_FLAGS)
	@echo "$(GREEN)✅ Clippy check passed!$(NC)"

.PHONY: audit
audit: ## Run security audit
	@echo "$(BLUE)🔒 Running security audit...$(NC)"
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-audit...$(NC)"; \
		cargo install cargo-audit; \
	fi
	cargo audit
	@echo "$(GREEN)✅ Security audit completed!$(NC)"

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@echo "$(BLUE)📦 Checking for outdated dependencies...$(NC)"
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-outdated...$(NC)"; \
		cargo install cargo-outdated; \
	fi
	cargo outdated

.PHONY: update
update: ## Update dependencies
	@echo "$(BLUE)📦 Updating dependencies...$(NC)"
	$(CARGO) update
	@echo "$(GREEN)✅ Dependencies updated!$(NC)"

.PHONY: upgrade
upgrade: ## Upgrade dependencies
	@echo "$(BLUE)📦 Upgrading dependencies...$(NC)"
	@if ! command -v cargo-upgrade >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-upgrade...$(NC)"; \
		cargo install cargo-upgrade; \
	fi
	cargo upgrade
	@echo "$(GREEN)✅ Dependencies upgraded!$(NC)"

# Docker targets
.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "$(BLUE)🐳 Building Docker image...$(NC)"
	@echo "$(CYAN)Image: $(DOCKER_IMAGE):$(DOCKER_TAG)$(NC)"
	$(DOCKER) build -f $(DOCKER_DIR)/Dockerfile -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	@echo "$(GREEN)✅ Docker image built!$(NC)"

.PHONY: docker-build-multi
docker-build-multi: ## Build multi-platform Docker image
	@echo "$(BLUE)🐳 Building multi-platform Docker image...$(NC)"
	$(DOCKER) buildx build --platform linux/amd64,linux/arm64 \
		-f $(DOCKER_DIR)/Dockerfile \
		-t $(DOCKER_IMAGE):$(DOCKER_TAG) \
		--push .
	@echo "$(GREEN)✅ Multi-platform Docker image built and pushed!$(NC)"

.PHONY: docker-run
docker-run: ## Run Docker container
	@echo "$(BLUE)🐳 Running Docker container...$(NC)"
	$(DOCKER) run --rm -it \
		--name $(PROJECT_NAME)-dev \
		-v $(PWD)/config:/app/config \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

.PHONY: docker-stop
docker-stop: ## Stop Docker container
	@echo "$(BLUE)🐳 Stopping Docker container...$(NC)"
	$(DOCKER) stop $(PROJECT_NAME)-dev 2>/dev/null || true
	@echo "$(GREEN)✅ Docker container stopped!$(NC)"

.PHONY: docker-clean
docker-clean: ## Clean Docker images
	@echo "$(BLUE)🐳 Cleaning Docker images...$(NC)"
	$(DOCKER) rmi $(DOCKER_IMAGE):$(DOCKER_TAG) 2>/dev/null || true
	@echo "$(GREEN)✅ Docker images cleaned!$(NC)"

# Docker Compose targets
.PHONY: up
up: ## Start services with docker-compose
	@echo "$(BLUE)🚀 Starting services...$(NC)"
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml up -d
	@echo "$(GREEN)✅ Services started!$(NC)"

.PHONY: down
down: ## Stop services with docker-compose
	@echo "$(BLUE)🛑 Stopping services...$(NC)"
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml down
	@echo "$(GREEN)✅ Services stopped!$(NC)"

.PHONY: logs
logs: ## View service logs
	@echo "$(BLUE)📋 Viewing service logs...$(NC)"
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml logs -f

.PHONY: restart
restart: ## Restart services
	@echo "$(BLUE)🔄 Restarting services...$(NC)"
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml restart
	@echo "$(GREEN)✅ Services restarted!$(NC)"

# Utility targets
.PHONY: install-tools
install-tools: ## Install development tools
	@echo "$(BLUE)🛠️ Installing development tools...$(NC)"
	$(RUSTUP) component add rustfmt clippy
	cargo install cargo-watch cargo-tarpaulin cargo-audit cargo-outdated cargo-upgrade
	@echo "$(GREEN)✅ Development tools installed!$(NC)"

.PHONY: check-tools
check-tools: ## Check if required tools are installed
	@echo "$(BLUE)🔍 Checking required tools...$(NC)"
	@command -v $(CARGO) >/dev/null 2>&1 || { echo "$(RED)❌ Cargo not found$(NC)"; exit 1; }
	@command -v $(RUSTC) >/dev/null 2>&1 || { echo "$(RED)❌ Rustc not found$(NC)"; exit 1; }
	@command -v $(DOCKER) >/dev/null 2>&1 || { echo "$(RED)❌ Docker not found$(NC)"; exit 1; }
	@echo "$(GREEN)✅ All required tools are available!$(NC)"

.PHONY: version
version: ## Show version information
	@echo "$(CYAN)Project: $(PROJECT_NAME)$(NC)"
	@echo "$(CYAN)Version: $(VERSION)$(NC)"
	@echo "$(CYAN)Git Commit: $(GIT_COMMIT)$(NC)"
	@echo "$(CYAN)Git Branch: $(GIT_BRANCH)$(NC)"
	@echo "$(CYAN)Build Time: $(BUILD_TIME)$(NC)"
	@echo "$(CYAN)Rust Version: $(shell $(RUSTC) --version)$(NC)"
	@echo "$(CYAN)Cargo Version: $(shell $(CARGO) --version)$(NC)"

.PHONY: size
size: ## Show binary size information
	@echo "$(BLUE)📏 Binary size information...$(NC)"
	@if [ -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ]; then \
		echo "$(CYAN)Release binary:$(NC)"; \
		ls -lh "$(TARGET_DIR)/release/$(BINARY_NAME)"; \
		echo ""; \
		echo "$(CYAN)Debug binary:$(NC)"; \
		ls -lh "$(TARGET_DIR)/debug/$(BINARY_NAME)" 2>/dev/null || echo "Debug binary not found"; \
	else \
		echo "$(YELLOW)No release binary found. Run 'make build' first.$(NC)"; \
	fi

.PHONY: config
config: ## Show current configuration
	@echo "$(BLUE)⚙️ Current configuration...$(NC)"
	@echo "$(CYAN)Environment: $(ENV)$(NC)"
	@echo "$(CYAN)Config file: $(CONFIG_FILE)$(NC)"
	@if [ -f "$(CONFIG_FILE)" ]; then \
		echo "$(GREEN)✅ Config file exists$(NC)"; \
		echo "$(CYAN)Config contents:$(NC)"; \
		cat "$(CONFIG_FILE)"; \
	else \
		echo "$(YELLOW)⚠️ Config file not found$(NC)"; \
		echo "$(CYAN)Available configs:$(NC)"; \
		ls -la "$(CONFIG_DIR)/"*.toml 2>/dev/null || echo "No config files found"; \
	fi

# Quality assurance targets
.PHONY: qa
qa: ## Run all quality checks
	@echo "$(BLUE)🔍 Running quality assurance checks...$(NC)"
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) test
	@$(MAKE) audit
	@echo "$(GREEN)✅ All quality checks passed!$(NC)"

.PHONY: pre-commit
pre-commit: ## Run pre-commit checks
	@echo "$(BLUE)🔍 Running pre-commit checks...$(NC)"
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) test
	@echo "$(GREEN)✅ Pre-commit checks passed!$(NC)"

.PHONY: ci
ci: ## Run CI pipeline locally
	@echo "$(BLUE)🚀 Running CI pipeline locally...$(NC)"
	@$(MAKE) clean
	@$(MAKE) check-all
	@$(MAKE) build-all
	@$(MAKE) test
	@$(MAKE) audit
	@$(MAKE) docker-build
	@echo "$(GREEN)✅ CI pipeline completed successfully!$(NC)"

# Release targets
.PHONY: release
release: ## Prepare release
	@echo "$(BLUE)🚀 Preparing release...$(NC)"
	@echo "$(CYAN)Version: $(VERSION)$(NC)"
	@$(MAKE) clean
	@$(MAKE) qa
	@$(MAKE) build
	@$(MAKE) docker-build
	@echo "$(GREEN)✅ Release prepared!$(NC)"
	@echo "$(YELLOW)Next steps:$(NC)"
	@echo "  1. git tag $(VERSION)"
	@echo "  2. git push origin $(VERSION)"
	@echo "  3. Create GitHub release"

.PHONY: release-all
release-all: ## Prepare release for all platforms
	@echo "$(BLUE)🚀 Preparing release for all platforms...$(NC)"
	@$(MAKE) clean
	@$(MAKE) qa
	@$(MAKE) build-all
	@$(MAKE) docker-build-multi
	@echo "$(GREEN)✅ Multi-platform release prepared!$(NC)"

# Documentation targets
.PHONY: docs
docs: ## Generate documentation
	@echo "$(BLUE)📚 Generating documentation...$(NC)"
	$(CARGO) doc --no-deps --open
	@echo "$(GREEN)✅ Documentation generated!$(NC)"

.PHONY: docs-check
docs-check: ## Check documentation
	@echo "$(BLUE)📚 Checking documentation...$(NC)"
	$(CARGO) doc --no-deps
	@echo "$(GREEN)✅ Documentation check completed!$(NC)"

# Benchmark targets
.PHONY: bench
bench: ## Run benchmarks
	@echo "$(BLUE)⚡ Running benchmarks...$(NC)"
	$(CARGO) bench
	@echo "$(GREEN)✅ Benchmarks completed!$(NC)"

# Profiling targets
.PHONY: profile
profile: ## Profile the application
	@echo "$(BLUE)📊 Profiling application...$(NC)"
	@if ! command -v cargo-flamegraph >/dev/null 2>&1; then \
		echo "$(YELLOW)Installing cargo-flamegraph...$(NC)"; \
		cargo install flamegraph; \
	fi
	cargo flamegraph --bin $(BINARY_NAME)
	@echo "$(GREEN)✅ Profiling completed!$(NC)"

# Default target
.DEFAULT_GOAL := help

# Print target info
.PHONY: info
info: ## Show build information
	@echo "$(CYAN)Build Information$(NC)"
	@echo "$(CYAN)================$(NC)"
	@echo "$(WHITE)Project:$(NC) $(PROJECT_NAME)"
	@echo "$(WHITE)Version:$(NC) $(VERSION)"
	@echo "$(WHITE)Environment:$(NC) $(ENV)"
	@echo "$(WHITE)Config:$(NC) $(CONFIG_FILE)"
	@echo "$(WHITE)Git:$(NC) $(GIT_COMMIT) on $(GIT_BRANCH)"
	@echo "$(WHITE)Build Time:$(NC) $(BUILD_TIME)"
	@echo "$(WHITE)Targets:$(NC) $(TARGETS)"
	@echo "$(WHITE)Rust:$(NC) $(shell $(RUSTC) --version)"
	@echo "$(WHITE)Cargo:$(NC) $(shell $(CARGO) --version)" 