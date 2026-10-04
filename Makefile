.PHONY: boot shell test console console-build

boot:
	cargo run -- boot

shell:
	cargo run -- shell

test:
	cargo test

console:
	cd console && npm run dev -- --host 0.0.0.0 --port 5173

console-build:
	cd console && npm run build
