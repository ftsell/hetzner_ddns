import requests
from requests.auth import AuthBase
from requests import PreparedRequest


HETZNER_URL = "https://dns.hetzner.com/api/v1"


class HetznerAuth(AuthBase):
    def __init__(self, api_token: str):
        self.api_token = api_token

    def __call__(self, r: PreparedRequest) -> PreparedRequest:
        r.headers["Auth-API-Token"] = self.api_token
        return r


class HetznerApi:
    def __init__(self, api_token: str):
        self.sess = requests.Session()
        self.sess.auth = HetznerAuth(api_token)

    def get_zone(self, zone_name: str):
        resp = self.sess.get(f"{HETZNER_URL}/zones?name={zone_name}")
        resp.raise_for_status()
        return resp.json()["zones"][0]

    def get_records(self, zone_id: str):
        resp = self.sess.get(f"{HETZNER_URL}/records?zone_id={zone_id}")
        resp.raise_for_status()
        return resp.json()["records"]

    def update_record(self, record_id: str, data: dict):
        resp = self.sess.put(f"{HETZNER_URL}/records/{record_id}", json=data)
        resp.raise_for_status()
        return resp.json()
