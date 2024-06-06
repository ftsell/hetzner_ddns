#!/usr/bin/env python3
import argparse
import tomllib
from pathlib import Path

from hetzner_ddns.hetzner_api import HetznerApi
from hetzner_ddns import kritzl_dev


def main():
    argp = argparse.ArgumentParser(prog="hetzner_ddns", description="DynDNS client for Hetzner DNS")
    argp.add_argument("-c", "--config", type=Path, required=True, help="Path to a config.toml")
    args = argp.parse_args()

    cfg = load_config(args.config)
    api = HetznerApi(cfg["api_token"])
    for target in cfg["targets"]:
        process_target(api, target)


def load_config(path: Path) -> dict:
    with open(path, "rb") as f:
        return tomllib.load(f)


def process_target(api: HetznerApi, target: dict):
    print(f"Processing target {target['record']}.{target['zone']}")
    zone = api.get_zone(target["zone"])
    records = (i for i in api.get_records(zone["id"]) if i["name"] == target["record"])
    for i_record in records:
        match i_record["type"]:
            case "A":
                new_value = kritzl_dev.get_ipv4()
            case "AAAA":
                new_value = kritzl_dev.get_ipv6()
            case _:
                continue

        print(f"Updating {i_record['type']} record to {new_value}")
        api.update_record(
            i_record["id"],
            {
                "name": i_record["name"],
                "ttl": 60,
                "type": i_record["type"],
                "value": new_value,
                "zone_id": i_record["zone_id"],
            },
        )


if __name__ == "__main__":
    main()
