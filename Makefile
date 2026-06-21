CARGO ?= cargo
NPM ?= npm
MDBOOK ?= mdbook
LYCHEE ?= lychee
DOCKER ?= docker
TARGET_ARG = $(if $(TARGET),--target $(TARGET),)

.PHONY: loadtest frontend-dev frontend-build frontend-clean site site-check site-serve \
	docker-build build-release version release-validate release-tag create-next-tag \
	ci-quality ci-test ci-audit ci-ui ci-docs ci-check

fe-dev: 
	cd ui && npm run dev

dev:
	@CMAKE_POLICY_VERSION_MINIMUM=3.5  cargo watch -x build

test:
	@mkdir -p ui/dist
	@CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo test -- --show-output

prod: frontend-build
	CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo build --release

run-prod: frontend-build
	@RUST_LOG=info ./target/release/event-gateway

run-postgres:
	@CMAKE_POLICY_VERSION_MINIMUM=3.5 APP_CONFIG_PATH=./configs/config-postgres.toml RUST_LOG=info cargo run

run-prod-postgres:
	@RUST_LOG=info APP_CONFIG_PATH=./configs/config-postgres.toml ./target/release/event-gateway


post_event:
	curl -v -X POST -H "Content-Type: application/json" -d '{ \
		"id": "123e4567-e89b-12d3-a456-426614174000", \
		"eventType": "user.click", \
		"eventVersion": "1.0", \
		"metadata": { \
			"key1": "value1", \
			"key2": "value2" \
		}, \
		"dataType": "string", \
		"data": { \
			"type": "json", \
			"content": { \
				"name": "example" \
			} \
		}, \
		"timestamp": "2023-01-28T12:00:00Z", \
		"origin": "localhost" \
	}' http://localhost:8080/api/v1/event

reloading:
	@systemfd --no-pid -s http::3000 -- cargo watch -x run

loadtest:
	@cargo run --release --manifest-path ./loadtest/Cargo.toml -- --report-file ./target/load_test.report.html --host http://localhost:8080 -u 1000 -r 1000 -t 60s

infra-run:
	@docker compose up -d

infra-stop:
	@docker compose stop

# Frontend targets
frontend-dev:
	cd ui && npm run dev

frontend-build:
	cd ui && $(NPM) ci && $(NPM) run build

frontend-clean:
	rm -rf ui/dist

clean: frontend-clean
	cargo clean

ci-quality:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --locked --workspace --all-targets -- -D warnings

ci-test:
	$(CARGO) test --locked --workspace --all-targets

ci-audit:
	cargo-audit audit

ci-ui:
	cd ui && $(NPM) ci && $(NPM) audit --omit=dev && $(NPM) run build

site:
	$(MDBOOK) build

site-check: site
	$(LYCHEE) --offline --no-progress --exclude-path 'book/404.html' book

site-serve:
	$(MDBOOK) serve --open

docker-build:
	$(DOCKER) build -t event-gateway:local .

build-release: frontend-build
	$(CARGO) build --release --locked -p event-gateway $(TARGET_ARG)

# Version management
version: ## Show current version
	@awk '/^\[package\]$$/ { in_package = 1; next } in_package && /^\[/ { exit } in_package && /^version = "/ { gsub(/^version = "|"/, ""); print; exit }' Cargo.toml

release-validate: ## Verify TAG matches the Cargo package version
	@test -n "$(TAG)" || { echo "TAG is required, for example TAG=v0.1.0"; exit 1; }
	@test "$(TAG)" = "v$$($(MAKE) --no-print-directory version)" || { \
		echo "Tag $(TAG) does not match Cargo.toml version v$$($(MAKE) --no-print-directory version)"; \
		exit 1; \
	}

release-tag: ## Create a local release tag from the Cargo package version
	@git diff --quiet && git diff --cached --quiet || { \
		echo "Working tree has tracked changes; commit them before tagging"; \
		exit 1; \
	}
	@tag="v$$($(MAKE) --no-print-directory version)"; \
	$(MAKE) --no-print-directory release-validate TAG="$$tag"; \
	git tag "$$tag"; \
	echo "Created $$tag; push it with: git push origin $$tag"

create-next-tag: ## Bump patch version, commit it, and create the matching tag
	@VERSION="$(VERSION)" ./scripts/create-next-tag.sh

ci-docs: site-check

ci-check: ci-quality ci-test ci-ui ci-docs
