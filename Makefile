.DEFAULT_GOAL := help

CARGO := cargo
MANIFEST := --manifest-path src-tauri/Cargo.toml

.PHONY: help dev build check test fmt fmt-check lint ci icons clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

dev: ## Run the app in development mode with hot reload
	$(CARGO) tauri dev

build: ## Build a release bundle for the current platform (macOS: ad-hoc signs it if no Developer ID is configured, since UNUserNotificationCenter requires a signed bundle)
	$(CARGO) tauri build
	@if [ "$$(uname)" = "Darwin" ]; then \
		APP=$$(find src-tauri/target/release/bundle/macos -maxdepth 1 -iname '*.app' 2>/dev/null | head -1); \
		if [ -n "$$APP" ]; then \
			if codesign -dv "$$APP" >/dev/null 2>&1; then \
				echo "$$APP is already signed, leaving it as-is"; \
			else \
				echo "Ad-hoc signing $$APP (no Developer ID configured — see README)"; \
				codesign --force --deep --sign - "$$APP"; \
			fi; \
		fi; \
	fi

check: ## Type-check the Rust code without building
	$(CARGO) check $(MANIFEST) --all-targets

test: ## Run the Rust test suite
	$(CARGO) test $(MANIFEST)

fmt: ## Format Rust code
	$(CARGO) fmt $(MANIFEST)

fmt-check: ## Check formatting without writing changes
	$(CARGO) fmt $(MANIFEST) -- --check

lint: ## Run Clippy with warnings denied
	$(CARGO) clippy $(MANIFEST) --all-targets -- -D warnings

ci: fmt-check lint test ## Run fmt-check, lint, and test (what CI should run)

icons: ## Regenerate the SF Symbols icon assets used by the settings UI (macOS only)
	swift scripts/export-sf-symbols.swift

clean: ## Remove build artifacts
	$(CARGO) clean $(MANIFEST)
