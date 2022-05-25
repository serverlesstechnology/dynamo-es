
configure:
	./db/create_tables.sh

test: configure
	cargo test

doc:
	cargo doc --lib --no-deps
