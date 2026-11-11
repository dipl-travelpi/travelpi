# Contributing

## Rules

1. `main` is protected. Every change goes through a pull request.
2. One issue, one branch, one pull request.
3. Pull requests are merged with squash once all checks pass. The branch is deleted afterwards.
4. Changes in a folder owned by someone else (see `.github/CODEOWNERS`) are agreed with the owner
   first. This applies in particular to `api/openapi.yaml`, the contract between all parts.
5. Never commit credentials: no tokens, passwords, Wi-Fi credentials or VPN keys, not even in
   test data. Examples use obviously made-up values.

## Names

| What | Form | Example |
|---|---|---|
| Branch | `TRA-<number>-<short-description>` | `TRA-42-tunnel-status` |
| Pull request title | `TRA-<number> <what it does>` | `TRA-42 Add tunnel status endpoint` |
| Commit message | imperative, capitalised, no full stop | `Add tunnel status endpoint` |

`TRA-<number>` is the number of the issue in the project's issue tracker. Code, comments, commit
messages and documentation are written in English.

## Size

A pull request should be readable in one sitting. Split larger work into several issues. Work in
progress can be opened as a draft pull request.

## Building

Binaries for the Raspberry Pi are built on a development computer or by CI, never on the device.
