watch-server:
	export RUST_LOG="info,wkd=trace"; cargo watch -x "run --bin wkd-tester-server"

test-cli-success:
	cargo run --package wkd-tester-cli -- --user-id "alexis.lowe@chimbosonic.com"

test-cli-failure:
	cargo run --package wkd-tester-cli -- --user-id "Joe.Doe@example.org"