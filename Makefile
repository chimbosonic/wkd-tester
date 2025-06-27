watch-server:
	cd server && cargo watch -x run

test-cli-success:
	cargo run --package wkd-tester-cli -- --user-id "alexis.lowe@chimbosonic.com"


test-cli-failure:
	cargo run --package wkd-tester-cli -- --user-id "Joe.Doe@example.org"