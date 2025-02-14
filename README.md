# WKD Tester

WKD tester is available in two variants:

- `wkd-tester` a CLI tool
- `wkd-tester-server` an API and Frontend

Both use the same library for testing and debugging [OpenPGP Web Key Directory (WKD)](https://datatracker.ietf.org/doc/draft-koch-openpgp-webkey-service/).

You can find `wkd-tester-server` hosted by `Alexis Lowe <alexis.lowe@chimbosonic.com>` at https://wkd.chimbosonic.com


## CLI: Usage

```bash
$ wkd-tester --help
A CLI tool for testing and debugging OpenPGP Web Key Directory (WKD)

Usage: wkd_tester --user-id <USER_ID>

Options:
  -u, --user-id <USER_ID>  The GPG User ID to look up (example: Joe.Doe@example.org)
  -h, --help               Print help
  -V, --version            Print version
```

## Server: Usage

TO-DO