import requests
import json
from urllib.parse import urljoin
from time import sleep

from tests.common.config import VictoriaMetricsConfig


def fetch_data(config: VictoriaMetricsConfig):
    export_url = urljoin(config.database_url, "api/v1/export")
    json_lines = []
    for line in requests.get(export_url, 'match[]={__name__!=""}').text.splitlines():
        json_lines.append(json.loads(line))
    json_lines.sort(key=lambda x: x['metric']['__name__'])
    return json_lines


def clear_data(config: VictoriaMetricsConfig):
    delete_url = urljoin(config.database_url,
                         "api/v1/admin/tsdb/delete_series")
    requests.post(delete_url, data={"match[]": '{__name__!=""}'})


def insert_data(config: VictoriaMetricsConfig, data):
    insert_url = urljoin(config.database_url,
                         "write")
    for line in data:
        requests.post(insert_url, line)
    sleep(2)  # Ensure that 'search.latencyOffset' passed
