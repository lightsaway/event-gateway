.PHONY: loadtest

dev:
	@cargo watch -x build

test:
	@cargo test -- --show-output

prod:
	@cargo build --release

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
	@cargo run --release --manifest-path ./loadtest/Cargo.toml -- --report-file ./target/load_test.report.html --host http://localhost:8080 -u 1000 -r 100 -t 60s

run-kafka:
	@docker run -d \
    --name redpanda \
    -p 9092:9092 \
    -p 9644:9644 \
    -e REDPANDA_AUTO_CREATE_TOPICS=true \
    vectorized/redpanda:latest \
    redpanda start \
    --overprovisioned \
    --node-id 0 \
    --kafka-addr 0.0.0.0:9092 \
    --advertise-kafka-addr 127.0.0.1:9092 \
    --check=true
