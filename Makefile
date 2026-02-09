UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    SED := $(shell command -v gsed 2>/dev/null)
    ifeq ($(SED),)
        $(error GNU sed (gsed) not found on macOS. \
			Install with: brew install gnu-sed)
    endif
else
    SED := sed
endif

.PHONY: help
help: ## Ask for help!
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; \
		{printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project in debug mode
	cargo build

.PHONY: build-release
build-release: ## Build the project in release mode
	cargo build --release

.PHONY: check
check: ## Check code for compilation errors
	cargo check --all-targets

.PHONY: check-format
check-format: ## Check code formatting
	cargo fmt --all -- --check

.PHONY: format
format: ## Format code
	cargo fmt --all

.PHONY: lint
lint: ## Run linter
	cargo clippy --all-targets -- -D warnings

.PHONY: test
test: ## Run tests
	cargo test

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: setup
setup: ## Setup development environment
	rustup component add rustfmt clippy

.PHONY: fix-trailing-whitespace
fix-trailing-whitespace: ## Remove trailing whitespaces from all files
	@echo "Removing trailing whitespaces from all files..."
	@find . -type f \( \
		-name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" \
		-o -name "*.yml" -o -name "*.ts" -o -name "*.tsx" \
		-o -name "*.js" -o -name "*.jsx" -o -name "*.sh" \
		-o -name "*.py" -o -name "*.go" -o -name "*.c" -o -name "*.h" \
		-o -name "*.cpp" -o -name "*.hpp" -o -name "*.json" \) \
		-not -path "./target/*" \
		-not -path "./node_modules/*" \
		-not -path "./.git/*" \
		-not -path "./dist/*" \
		-not -path "./build/*" \
		-exec sh -c \
			'echo "Processing: $$1"; $(SED) -i -e "s/[[:space:]]*$$//" "$$1"' \
			_ {} \; && \
		echo "Trailing whitespaces removed."

.PHONY: check-trailing-whitespace
check-trailing-whitespace: ## Check for trailing whitespaces in source files
	@echo "Checking for trailing whitespaces..."
	@files_with_trailing_ws=$$(find . -type f \( \
		-name "*.rs" -o -name "*.toml" -o -name "*.md" -o -name "*.yaml" \
		-o -name "*.yml" -o -name "*.ts" -o -name "*.tsx" \
		-o -name "*.js" -o -name "*.jsx" -o -name "*.sh" \
		-o -name "*.py" -o -name "*.go" -o -name "*.c" -o -name "*.h" \
		-o -name "*.cpp" -o -name "*.hpp" -o -name "*.json" \) \
		-not -path "./target/*" \
		-not -path "./node_modules/*" \
		-not -path "./.git/*" \
		-not -path "./dist/*" \
		-not -path "./build/*" \
		-not -path "./doc/node_modules/*" \
		-not -path "./doc/build/*" \
		-not -path "./doc/.docusaurus/*" \
		-exec grep -l '[[:space:]]$$' {} + 2>/dev/null || true); \
	if [ -n "$$files_with_trailing_ws" ]; then \
		echo "Files with trailing whitespaces found:"; \
		echo "$$files_with_trailing_ws" | sed 's/^/  /'; \
		echo ""; \
		echo "Run 'make fix-trailing-whitespace' to fix automatically."; \
		exit 1; \
	else \
		echo "No trailing whitespaces found."; \
	fi

# Documentation targets

.PHONY: doc-install
doc-install: ## Install documentation dependencies
	npm --prefix doc install

.PHONY: doc-dev
doc-dev: ## Run documentation dev server
	(cd doc && npx docusaurus start)

.PHONY: doc-build
doc-build: ## Build documentation for production
	(cd doc && npx docusaurus build)

.PHONY: doc-serve
doc-serve: ## Serve built documentation locally
	(cd doc && npx docusaurus serve)

.PHONY: doc-clear
doc-clear: ## Clear documentation cache
	(cd doc && npx docusaurus clear)

# E2E test targets

.PHONY: e2e-install
e2e-install: ## Install E2E test dependencies
	cd e2e && npm install && npx playwright install

.PHONY: e2e-test
e2e-test: ## Run E2E tests
	cd e2e && npx playwright test

.PHONY: e2e-test-ui
e2e-test-ui: ## Run E2E tests with UI
	cd e2e && npx playwright test --ui

.PHONY: e2e-test-headed
e2e-test-headed: ## Run E2E tests in headed mode
	cd e2e && npx playwright test --headed

.PHONY: e2e-report
e2e-report: ## Show E2E test report
	cd e2e && npx playwright show-report

# Publish targets
# Crates must be published in dependency order.
# Sleep between publishes to let the crates.io index update.

PUBLISH_CRATES := \
	oxide-sql-derive \
	oxide-sql-core \
	oxide-router \
	oxide-sql-sqlite \
	oxide-orm \
	oxide-forms \
	oxide-migrate \
	oxide-auth \
	oxide-admin

.PHONY: publish
publish: ## Publish all crates to crates.io
	@for crate in $(PUBLISH_CRATES); do \
		echo "Publishing $$crate..."; \
		cargo publish -p $$crate || exit 1; \
		echo "Waiting for crates.io index to update..."; \
		sleep 30; \
	done
	@echo "All crates published successfully"

.PHONY: publish-dry-run
publish-dry-run: ## Dry-run publish for all crates
	cargo package --workspace --allow-dirty
	@echo "Dry-run complete — all crates are ready to publish"

# Example targets

.PHONY: example-blog
example-blog: ## Run blog admin example
	cargo run --example blog_admin
