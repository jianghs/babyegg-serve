up:
	docker compose up -d

down:
	docker compose down

check:
	cargo check --workspace

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace

run-blog-api:
	cargo run -p blog-api
