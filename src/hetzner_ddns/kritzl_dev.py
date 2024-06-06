from functools import cache
import requests

@cache
def get_ipv4() -> str:
    resp = requests.get("https://4.kritzl.dev")
    resp.raise_for_status()
    return resp.text


@cache
def get_ipv6():
    resp = requests.get("https://6.kritzl.dev")
    resp.raise_for_status()
    return resp.text
