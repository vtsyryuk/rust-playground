#!/usr/bin/env python3
"""Check a deployed liquidity-hub demo, including a bounded cold-start wait."""
import os
import time
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import urlopen


def verify(base_url, expected_ticks=100, timeout=300):
    parsed = urlsplit(base_url)
    if (parsed.scheme not in {"http", "https"} or not parsed.hostname
            or parsed.username or parsed.password or parsed.query or parsed.fragment):
        raise ValueError("Provide a plain HTTP(S) deployment URL without credentials, query or fragment")
    base_url = base_url.rstrip("/")
    deadline = time.monotonic() + timeout
    while True:
        try:
            with urlopen(base_url + "/health", timeout=15) as response:
                if response.status != 200 or response.read().decode().strip() != "ok":
                    raise ValueError("Unexpected health response")
            break
        except (HTTPError, URLError, TimeoutError, ValueError):
            if time.monotonic() >= deadline:
                raise RuntimeError("Cloud demo did not become healthy before the deadline") from None
            time.sleep(min(5, max(0, deadline - time.monotonic())))
    with urlopen(base_url + "/ready", timeout=15) as response:
        expected = "processed=" + str(expected_ticks)
        if response.status != 200 or response.read().decode().strip() != expected:
            raise RuntimeError("Readiness did not report the expected processed mock ticks")
    try:
        with urlopen(base_url + "/e2e-unknown-route", timeout=15):
            raise RuntimeError("Unknown routes must return HTTP 404")
    except HTTPError as error:
        if error.code != 404:
            raise RuntimeError("Unknown route returned an unexpected status") from None
    print("PASS: healthy demo, expected processed ticks, and unknown-route handling")


def main():
    base_url = os.environ.get("CLOUD_BASE_URL", "")
    if not base_url:
        raise SystemExit("Deploy render.yaml and set CLOUD_BASE_URL, or supply base_url. No cloud test ran.")
    expected = int(os.environ.get("CLOUD_EXPECTED_TICKS", "100"))
    if expected < 1:
        raise SystemExit("CLOUD_EXPECTED_TICKS must be positive")
    verify(base_url, expected)


if __name__ == "__main__":
    main()
