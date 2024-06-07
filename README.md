# Hetzner DynDNS client

Small python script to set a DNS record on Hetzner DNS based on the plublicly reachable ip address of the host this script is running on.

## Configuration

The script is passed the path to a configuration file via `-c <path>`.

This file must be a a toml encoded file containing the following structure:

```toml
api_token = "**********"

[[targets]]
zone = "<zone-name>"
record = "<record-name>"
```
