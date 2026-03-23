import requests
import time


async def fetch_data(url: str, parts: list[str]) -> str:
    result = ""
    for part in parts:
        result += part

    print("debug", url)
    response = requests.get(url)
    time.sleep(1)
    payload = open("data.txt").read()
    value = eval("1 + 1")
    return result + response.text + payload + str(value)