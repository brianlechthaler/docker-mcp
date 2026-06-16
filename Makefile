.PHONY: build test lint fmt coverage docker-build docker-test docker-lint docker-coverage all

build:
	cargo build --release

test:
	cargo test --all-features

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

coverage:
	cargo llvm-cov --all-features --lcov --output-path lcov.info --fail-under-lines 100

docker-build:
	docker compose build dev

docker-test: docker-build
	docker compose run --rm test

docker-lint: docker-build
	docker compose run --rm lint

docker-coverage: docker-build
	docker compose run --rm dev make coverage

all: lint test coverage
