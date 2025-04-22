.PHONY: loadtest frontend-dev frontend-build frontend-clean

dev: frontend-dev
	@cargo watch -x build

test:
	@mkdir -p ui/dist
	@CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo test -- --show-output

prod: frontend-build
	CMAKE_POLICY_VERSION_MINIMUM=3.5 cargo build --release

run-prod: frontend-build
	@RUST_LOG=info ./target/release/event-gateway

run-postgres:
	@CMAKE_POLICY_VERSION_MINIMUM=3.5 APP_CONFIG_PATH=config-postgres.toml RUST_LOG=info cargo run

run-prod-postgres:
	@RUST_LOG=info APP_CONFIG_PATH=config-postgres.toml ./target/release/event-gateway


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
	cd ui && npm install && npm run build

frontend-clean:
	rm -rf ui/dist

clean: frontend-clean
	cargo clean
