# API

`openapi.yaml` describes the HTTP API that `travelpid` offers on its Unix socket. It is the
contract between the daemon, the dashboard and the command-line client, and it is changed
before the code.

## Version 0

| Path | What it does |
|---|---|
| `GET /v0/status` | The whole state at once, one section per module |
| `GET /v0/events` | Server-Sent Events: the current state of every section on connect, then the changed section on every change |

| Section | Module | Content |
|---|---|---|
| `ap` | own Wi-Fi | interface, state, SSID, channel, subnet, number of devices, error |
| `uplink` | internet source | interface, state, SSID, signal, IPv4 address, internet reachable, error |

A section has the same fields in both paths. Fields without a value are `null`, never missing.
No response contains secrets.

## Changing the API

1. Change `openapi.yaml` first, in the same pull request as the code or before it.
2. If the change touches another person's module, agree it with that person first.
3. Check the file:

   ```sh
   npx @redocly/cli@1 lint api/openapi.yaml
   ```

   The file must be valid. Four warnings are expected and accepted:

   | Warning | Why it stays |
   |---|---|
   | `info-license` | The license of the project is not decided yet |
   | `no-server-example.com` | `localhost` is correct: the host name is ignored on the Unix socket |
   | `operation-4xx-response` (twice) | The two `GET` paths have no input that could be wrong |

## Reading it

Any OpenAPI viewer works, for example:

```sh
npx @redocly/cli@1 preview-docs api/openapi.yaml
```
