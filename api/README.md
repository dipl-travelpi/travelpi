# API

`openapi.yaml` in this folder will describe the HTTP API that `travelpid` offers on its Unix
socket: reading status, sending commands and the event stream. It is the contract between the
daemon, the dashboard and the command-line client, and it is written before the code.
