import requests
import json
from urllib.parse import urljoin
from time import sleep


class VictoriaMetricsConfig:
    def __init__(self, database_url="http://localhost:8428"):
        self.database_url = database_url

    def to_dict(self):
        return {
            "VICTORIA_METRICS_OUTPUT_URL": self.database_url,
            "VICTORIA_QUERY_URL": urljoin(self.database_url, '/api/v1'),
        }


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
