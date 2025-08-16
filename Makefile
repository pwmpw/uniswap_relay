# Uniswap Relay DApp Makefile
# A robust build system for development, testing, and deployment

# Configuration
PROJECT_NAME := uniswap_relay
BINARY_NAME := uniswap_relay
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
DOCKER_COMPOSE := docker compose
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

# Help target
.PHONY: help
help: ## Show this help message
	@echo "Uniswap Relay DApp - Available Targets"
	@echo "====================================="
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "%-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Environment Variables:"
	@echo "  ENV=development|production  Configuration environment (default: development)"
	@echo "  VERSION=tag                 Override version (default: git tag)"
	@echo ""
	@echo "Examples:"
	@echo "  make build ENV=production"
	@echo "  make test VERSION=v1.0.0"
	@echo "  make docker-build DOCKER_TAG=latest"

# Development targets
.PHONY: dev
dev: ## Run in development mode
	@echo "🚀 Starting development mode..."
	@echo "Environment: $(ENVIRONMENT)"
	@echo "Config: $(CONFIG_FILE)"
	ENVIRONMENT=$(ENVIRONMENT) $(CARGO) run --bin $(BINARY_NAME)

.PHONY: run
run: ## Run the built binary
	@echo "🚀 Running binary..."
	@echo "Environment: $(ENVIRONMENT)"
	@echo "Config: $(CONFIG_FILE)"
	@if [ ! -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ]; then \
		echo "Binary not found. Building first..."; \
		$(MAKE) build; \
	fi
	ENVIRONMENT=$(ENVIRONMENT) ./$(TARGET_DIR)/release/$(BINARY_NAME)

.PHONY: dev-watch
dev-watch: ## Run with cargo watch for auto-reload
	@echo "👀 Starting development mode with auto-reload..."
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	cargo watch -x "run --bin $(BINARY_NAME)"

.PHONY: check
check: ## Check code without building
	@echo "🔍 Checking code..."
	$(CARGO) check --all-features

.PHONY: check-all
check-all: ## Check code for all targets
	@echo "🔍 Checking code for all targets..."
	@for target in $(TARGETS); do \
		echo "Checking $$target..."; \
		$(CARGO) check --target $$target --all-features || exit 1; \
	done

# Building targets
.PHONY: build
build: ## Build release binary
	@echo "🔨 Building release binary..."
	@echo "Version: $(VERSION)"
	@echo "Commit: $(GIT_COMMIT)"
	@echo "Branch: $(GIT_BRANCH)"
	@echo "Build time: $(BUILD_TIME)"
	$(CARGO) build $(CARGO_BUILD_FLAGS) $(CARGO_FLAGS)
	@echo "✅ Build completed!"
	@echo "Binary location: $(TARGET_DIR)/release/$(BINARY_NAME)"
	@ls -lh $(TARGET_DIR)/release/$(BINARY_NAME)

.PHONY: build-debug
build-debug: ## Build debug binary
	@echo "🔨 Building debug binary..."
	$(CARGO) build $(CARGO_FLAGS)
	@echo "✅ Debug build completed!"

.PHONY: build-all
build-all: ## Build for all supported targets
	@echo "🔨 Building for all targets..."
	@for target in $(TARGETS); do \
		echo "Building for $$target..."; \
		$(CARGO) build --target $$target $(CARGO_BUILD_FLAGS) $(CARGO_FLAGS) || exit 1; \
	done
	@echo "✅ All targets built successfully!"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf $(COVERAGE_DIR)
	@echo "✅ Clean completed!"

.PHONY: clean-all
clean-all: ## Clean all artifacts including dependencies
	@echo "🧹 Cleaning all artifacts..."
	$(CARGO) clean
	@rm -rf $(COVERAGE_DIR)
	@rm -rf target/
	@rm -rf Cargo.lock
	@echo "✅ Complete clean finished!"

# Testing targets
.PHONY: test
test: ## Run all tests
	@echo "🧪 Running tests..."
	$(CARGO) test $(CARGO_TEST_FLAGS) $(CARGO_FLAGS)
	@echo "✅ Tests completed!"

.PHONY: test-unit
test-unit: ## Run unit tests only
	@echo "🧪 Running unit tests..."
	$(CARGO) test --lib $(CARGO_FLAGS)
	@echo "✅ Unit tests completed!"

.PHONY: test-integration
test-integration: ## Run integration tests only
	@echo "🧪 Running integration tests..."
	$(CARGO) test --test integration $(CARGO_FLAGS)
	@echo "✅ Integration tests completed!"

.PHONY: test-watch
test-watch: ## Run tests with auto-reload
	@echo "👀 Running tests with auto-reload..."
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "Installing cargo-watch..."; \
		cargo install cargo-watch; \
	fi
	cargo watch -x "test $(CARGO_TEST_FLAGS)"

.PHONY: test-coverage
test-coverage: ## Run tests with coverage report
	@echo "📊 Running tests with coverage..."
	@if ! command -v cargo-tarpaulin >/dev/null 2>&1; then \
		echo "Installing cargo-tarpaulin..."; \
		cargo install cargo-tarpaulin; \
	fi
	@mkdir -p $(COVERAGE_DIR)
	cargo tarpaulin --out Html --output-dir $(COVERAGE_DIR)
	@echo "✅ Coverage report generated in $(COVERAGE_DIR)/"

# Code quality targets
.PHONY: fmt
fmt: ## Format code
	@echo "🎨 Formatting code..."
	$(CARGO) fmt --all
	@echo "✅ Code formatting completed!"

.PHONY: fmt-check
fmt-check: ## Check code formatting
	@echo "🎨 Checking code formatting..."
	$(CARGO) fmt $(CARGO_FMT_FLAGS) $(CARGO_FLAGS)
	@echo "✅ Code formatting check passed!"

.PHONY: clippy
clippy: ## Run clippy linter (strict)
	@echo "🔍 Running clippy..."
	$(CARGO) clippy $(CARGO_CLIPPY_FLAGS) $(CARGO_FLAGS)
	@echo "✅ Clippy check passed!"

.PHONY: clippy-check
clippy-check: ## Run clippy linter (warnings only)
	@echo "🔍 Running clippy (warnings only)..."
	$(CARGO) clippy --all-targets --all-features
	@echo "✅ Clippy check completed!"

.PHONY: actionlint
actionlint: ## Validate GitHub Actions syntax
	@echo "🔍 Validating GitHub Actions syntax..."
	@if ! command -v actionlint >/dev/null 2>&1; then \
		echo "Installing actionlint..."; \
		if command -v brew >/dev/null 2>&1; then \
			brew install actionlint; \
		elif command -v go >/dev/null 2>&1; then \
			go install github.com/rhysd/actionlint/cmd/actionlint@latest; \
		else \
			echo "Error: Cannot install actionlint. Please install manually:"; \
			echo "  - macOS: brew install actionlint"; \
			echo "  - Linux: go install github.com/rhysd/actionlint/cmd/actionlint@latest"; \
			exit 1; \
		fi; \
	fi
	actionlint
	@echo "✅ GitHub Actions syntax validation passed!"

.PHONY: audit
audit: ## Run security audit
	@echo "🔒 Running security audit..."
	@if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "Installing cargo-audit..."; \
		cargo install cargo-audit --version 0.21.2; \
	fi
	cargo audit
	@echo "✅ Security audit completed!"

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@echo "📦 Checking for outdated dependencies..."
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "Installing cargo-outdated..."; \
		cargo install cargo-outdated; \
	fi
	cargo outdated

.PHONY: update
update: ## Update dependencies
	@echo "📦 Updating dependencies..."
	$(CARGO) update
	@echo "✅ Dependencies updated!"

.PHONY: upgrade
upgrade: ## Upgrade dependencies
	@echo "📦 Upgrading dependencies..."
	@if ! command -v cargo-upgrade >/dev/null 2>&1; then \
		echo "Installing cargo-upgrade..."; \
		cargo install cargo-upgrade; \
	fi
	cargo upgrade
	@echo "✅ Dependencies upgraded!"

# Docker targets
.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "🐳 Building Docker image..."
	@echo "Image: $(DOCKER_IMAGE):$(DOCKER_TAG)"
	$(DOCKER) build -f $(DOCKER_DIR)/Dockerfile -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	@echo "✅ Docker image built!"

.PHONY: docker-build-multi
docker-build-multi: ## Build multi-platform Docker image
	@echo "🐳 Building multi-platform Docker image..."
	$(DOCKER) buildx build --platform linux/amd64,linux/arm64 \
		-f $(DOCKER_DIR)/Dockerfile \
		-t $(DOCKER_IMAGE):$(DOCKER_TAG) \
		--push .
	@echo "✅ Multi-platform Docker image built and pushed!"

.PHONY: docker-run
docker-run: ## Run Docker container
	@echo "🐳 Running Docker container..."
	$(DOCKER) run --rm -it \
		--name $(PROJECT_NAME)-dev \
		-v $(PWD)/config:/app/config \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

.PHONY: docker-stop
docker-stop: ## Stop Docker container
	@echo "🐳 Stopping Docker container..."
	$(DOCKER) stop $(PROJECT_NAME)-dev 2>/dev/null || true
	@echo "✅ Docker container stopped!"

.PHONY: docker-clean
docker-clean: ## Clean Docker images
	@echo "🐳 Cleaning Docker images..."
	$(DOCKER) rmi $(DOCKER_IMAGE):$(DOCKER_TAG) 2>/dev/null || true
	@echo "✅ Docker images cleaned!"

# Docker Compose targets
.PHONY: up
up: ## Start services with docker-compose
	@echo "🚀 Starting services..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml up -d
	@echo "✅ Services started!"

.PHONY: down
down: ## Stop services with docker-compose
	@echo "🛑 Stopping services..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml down
	@echo "✅ Services stopped!"

# Production targets
.PHONY: production-up
production-up: ## Start production services
	@echo "🚀 Starting production services..."
	@echo "Environment: production"
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.production.yml up -d
	@echo "✅ Production services started!"
	@echo "Access monitoring:"
	@echo "  - Grafana: http://localhost:3000 (admin/admin)"
	@echo "  - Prometheus: http://localhost:9090"
	@echo "  - AlertManager: http://localhost:9093"

.PHONY: production-down
production-down: ## Stop production services
	@echo "🛑 Stopping production services..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.production.yml down
	@echo "✅ Production services stopped!"

.PHONY: production-logs
production-logs: ## View production logs
	@echo "📋 Production logs..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.production.yml logs -f

.PHONY: production-status
production-status: ## Check production service status
	@echo "📊 Production service status..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.production.yml ps

.PHONY: deploy-production
deploy-production: ## Deploy to production using GitHub Actions
	@echo "🚀 Deploying to production..."
	@echo "This will trigger the GitHub Actions deployment workflow"
	@echo "Make sure you have:"
	@echo "  1. The Graph API key configured"
	@echo "  2. Production Redis instance ready"
	@echo "  3. GitHub repository secrets configured"
	@echo ""
	@echo "To deploy:"
	@echo "  1. Go to Actions → Deploy to Production"
	@echo "  2. Click 'Run workflow'"
	@echo "  3. Select environment: production"
	@echo "  4. Enter version tag"
	@echo "  5. Click 'Run workflow'"

.PHONY: logs
logs: ## View service logs
	@echo "📋 Viewing service logs..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml logs -f

.PHONY: restart
restart: ## Restart services
	@echo "🔄 Restarting services..."
	$(DOCKER_COMPOSE) -f $(DOCKER_DIR)/docker-compose.yml restart
	@echo "✅ Services restarted!"

# Utility targets
.PHONY: install-tools
install-tools: ## Install development tools
	@echo "🛠️ Installing development tools..."
	$(RUSTUP) component add rustfmt clippy
	cargo install cargo-watch cargo-tarpaulin cargo-audit --version 0.18 cargo-outdated cargo-upgrade
	@echo "✅ Development tools installed!"

.PHONY: check-tools
check-tools: ## Check if required tools are installed
	@echo "🔍 Checking required tools..."
	@command -v $(CARGO) >/dev/null 2>&1 || { echo "❌ Cargo not found"; exit 1; }
	@command -v $(RUSTC) >/dev/null 2>&1 || { echo "❌ Rustc not found"; exit 1; }
	@command -v $(DOCKER) >/dev/null 2>&1 || { echo "❌ Docker not found"; exit 1; }
	@echo "✅ All required tools are available!"

.PHONY: version
version: ## Show version information
	@echo "Project: $(PROJECT_NAME)"
	@echo "Version: $(VERSION)"
	@echo "Git Commit: $(GIT_COMMIT)"
	@echo "Git Branch: $(GIT_BRANCH)"
	@echo "Build Time: $(BUILD_TIME)"
	@echo "Rust Version: $(shell $(RUSTC) --version)"
	@echo "Cargo Version: $(shell $(CARGO) --version)"

.PHONY: size
size: ## Show binary size information
	@echo "📏 Binary size information..."
	@if [ -f "$(TARGET_DIR)/release/$(BINARY_NAME)" ]; then \
		echo "Release binary:"; \
		ls -lh "$(TARGET_DIR)/release/$(BINARY_NAME)"; \
		echo ""; \
		echo "Debug binary:"; \
		ls -lh "$(TARGET_DIR)/debug/$(BINARY_NAME)" 2>/dev/null || echo "Debug binary not found"; \
	else \
		echo "No release binary found. Run 'make build' first."; \
	fi

.PHONY: config
config: ## Show current configuration
	@echo "⚙️ Current configuration..."
	@echo "Environment: $(ENV)"
	@echo "Config file: $(CONFIG_FILE)"
	@if [ -f "$(CONFIG_FILE)" ]; then \
		echo "✅ Config file exists"; \
		echo "Config contents:"; \
		cat "$(CONFIG_FILE)"; \
	else \
		echo "⚠️ Config file not found"; \
		echo "Available configs:"; \
		ls -la "$(CONFIG_DIR)/"*.toml 2>/dev/null || echo "No config files found"; \
	fi

# Quality assurance targets
.PHONY: qa
qa: ## Run all quality checks
	@echo "🔍 Running quality assurance checks..."
	@$(MAKE) fmt-check
	@$(MAKE) clippy-check
	@$(MAKE) actionlint
	@$(MAKE) test
	@$(MAKE) audit
	@echo "✅ All quality checks passed!"

.PHONY: pre-commit
pre-commit: ## Run pre-commit checks
	@echo "🔍 Running pre-commit checks..."
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) actionlint
	@$(MAKE) test
	@echo "✅ Pre-commit checks passed!"

.PHONY: ci
ci: ## Run CI pipeline locally
	@echo "🚀 Running CI pipeline locally..."
	@$(MAKE) clean
	@$(MAKE) check-all
	@$(MAKE) build-all
	@$(MAKE) test
	@$(MAKE) audit
	@$(MAKE) docker-build
	@echo "✅ CI pipeline completed successfully!"

# Release targets
.PHONY: release
release: ## Prepare release
	@echo "🚀 Preparing release..."
	@echo "Version: $(VERSION)"
	@$(MAKE) clean
	@$(MAKE) qa
	@$(MAKE) build
	@$(MAKE) docker-build
	@echo "✅ Release prepared!"
	@echo "Next steps:"
	@echo "  1. git tag $(VERSION)"
	@echo "  2. git push origin $(VERSION)"
	@echo "  3. Create GitHub release"

.PHONY: release-all
release-all: ## Prepare release for all platforms
	@echo "🚀 Preparing release for all platforms..."
	@$(MAKE) clean
	@$(MAKE) qa
	@$(MAKE) build-all
	@$(MAKE) docker-build-multi
	@echo "✅ Multi-platform release prepared!"

# Documentation targets
.PHONY: docs
docs: ## Generate documentation
	@echo "📚 Generating documentation..."
	$(CARGO) doc --no-deps --open
	@echo "✅ Documentation generated!"

.PHONY: docs-check
docs-check: ## Check documentation
	@echo "📚 Checking documentation..."
	$(CARGO) doc --no-deps
	@echo "✅ Documentation check completed!"

# Benchmark targets
.PHONY: bench
bench: ## Run benchmarks
	@echo "⚡ Running benchmarks..."
	$(CARGO) bench
	@echo "✅ Benchmarks completed!"

# Profiling targets
.PHONY: profile
profile: ## Profile the application
	@echo "📊 Profiling application..."
	@if ! command -v cargo-flamegraph >/dev/null 2>&1; then \
		echo "Installing cargo-flamegraph..."; \
		cargo install flamegraph; \
	fi
	cargo flamegraph --bin $(BINARY_NAME)
	@echo "✅ Profiling completed!"

# Default target
.DEFAULT_GOAL := help

# Print target info
.PHONY: info
info: ## Show build information
	@echo "Build Information"
	@echo "================"
	@echo "Project: $(PROJECT_NAME)"
	@echo "Version: $(VERSION)"
	@echo "Environment: $(ENV)"
	@echo "Config: $(CONFIG_FILE)"
	@echo "Git: $(GIT_COMMIT) on $(GIT_BRANCH)"
	@echo "Build Time: $(BUILD_TIME)"
	@echo "Targets: $(TARGETS)"
	@echo "Rust: $(shell $(RUSTC) --version)"
	@echo "Cargo: $(shell $(CARGO) --version)" 