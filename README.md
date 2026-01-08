# WKD Tester

WKD tester is available in two variants:

- `wkd-tester` a CLI tool
- `wkd-tester-server` an API and Frontend

Both use the same library for testing and debugging [OpenPGP Web Key Directory (WKD)](https://datatracker.ietf.org/doc/draft-koch-openpgp-webkey-service/).

You can find `wkd-tester-server` hosted by `Alexis Lowe <alexis.lowe@chimbosonic.com>` at https://wkd.dp42.dev

## CLI: Install

Clone this repo and go to the folder then run:

```bash
$ cargo install --path cli
```

## CLI: Usage

```bash
$ wkd-tester --help
A CLI tool for testing and debugging OpenPGP Web Key Directory (WKD)

Usage: wkd-tester --user-id <USER_ID>

Options:
  -u, --user-id <USER_ID>  The GPG User ID to look up (example: Joe.Doe@example.org)
  -h, --help               Print help
  -V, --version            Print version
```

### CLI: Usage Example

No Errors example:
```bash
$ wkd-tester -u alexis.lowe@chimbosonic.com
Advanced method URI: https://openpgpkey.chimbosonic.com/.well-known/openpgpkey/chimbosonic.com/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Direct method URI: https://chimbosonic.com/.well-known/openpgpkey/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Advanced method fetch was successful
Advanced method key loading succeed with fingerprint: AC48BC1F029B6188D97E2D807C855DB4466DF0C6
Direct method fetch was successful
Direct method key loading succeed with fingerprint: AC48BC1F029B6188D97E2D807C855DB4466DF0C6
```

Warnings example:
```bash
$ wkd-tester -u alexis.lowe@chimbosonic.com
Advanced method URI: https://openpgpkey.chimbosonic.com/.well-known/openpgpkey/chimbosonic.com/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Direct method URI: https://chimbosonic.com/.well-known/openpgpkey/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Advanced method fetch was successful with warnings:
wkd_fetch

  ! Content-Type header is not set to 'application/octet-stream'. This may cause issues
  | with parsing

wkd_fetch

  ! Access-Control-Allow-Origin header is not set to '*'. This may cause issues with
  | CORS

Advanced method key loading succeed with fingerprint: AC48BC1F029B6188D97E2D807C855DB4466DF0C6
Direct method fetch was successful with warnings:
wkd_fetch

  ! Content-Type header is not set to 'application/octet-stream'. This may cause issues
  | with parsing

wkd_fetch

  ! Access-Control-Allow-Origin header is not set to '*'. This may cause issues with
  | CORS

Direct method key loading succeed with fingerprint: AC48BC1F029B6188D97E2D807C855DB4466DF0C6
```

Errors example:
```bash
$ wkd-tester -u alexis.lowe@example.org
Advanced method URI: https://openpgpkey.example.org/.well-known/openpgpkey/example.org/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Direct method URI: https://example.org/.well-known/openpgpkey/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe
Advanced method fetch failed with error:
wkd_fetch

  x WKD URI provided is not a valid URL
  |-> error sending request for url (https://openpgpkey.example.org/.well-known/
  |   openpgpkey/example.org/hu/z9naq3iddua5t55b3hp1w3hwz8eyrc7n?l=alexis.lowe)
  |-> client error (Connect)
  |-> dns error: failed to lookup address information: Name or service not known
  `-> failed to lookup address information: Name or service not known

Direct method fetch failed with error:
wkd_fetch

  x Status code is not 200
```

## Server: Usage

Clone the repo, make sure to update `server/src/config.rs`

### CAUTION

- `root_path` does **not** change server routing. It only affects paths referenced in generated/static content.
- `root_path` must **not** end with `/`.
  - Use `""` for the site root.
  - If you proxy the server under a subpath (e.g. `https://example.org/wkd`), set `root_path` to `/wkd`.

**Swagger UI note:** If `root_path != ""`, the Swagger UI page will break because it hardcodes `/api-docs/openapi.json`. I wasn’t able to get `utoipa` to change the path used in the frontend.


and then build the server container:
```bash
docker build -t wkd-tester .
```

Then run it with:
```bash
docker run -it --rm -p 7070:7070 wkd-tester
```

## Tracing
To enable tracing for the `wkd` library run the server with the following:
```bash
cd server/
export RUST_LOG="info,wkd=trace"; cargo watch -x run --features otel
```
and in another terminal:
```bash
docker run  -p 4317:4317 -p 16686:16686 jaegertracing/all-in-one:latest
```