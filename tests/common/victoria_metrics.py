import requests
import json
from urllib.parse import urljoin

from tests.common.config import VictoriaMetricsConfig


class VictoriaMetrics:
    def __init__(self, victoria_metrics_config: VictoriaMetricsConfig):
        self.config = victoria_metrics_config

    def fetch_data_table(self):
        export_url = urljoin(self.config.database_url, "api/v1/export")
        json_lines = []
        for line in requests.get(export_url, 'match[]={__name__!=""}').text.splitlines():
            json_lines.append(json.loads(line))
        return json_lines

    def clear_data_base(self):
        delete_url = urljoin(self.config.database_url,
                             "api/v1/admin/tsdb/delete_series")
        requests.post(delete_url, data={"match[]": '{__name__!=""}'})

    def is_db_alive(self):
        result = requests.get(self.config.database_url)
        result.raise_for_status()
        return result.ok
