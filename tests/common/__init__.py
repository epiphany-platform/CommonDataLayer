import json
import os
import time


def load_case(case_name, app):
    with open(os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'data', app, f'{case_name}.json')) as file:
        json_document = json.load(file)
        return json_document['data'], json_document['expected']


def retry_retrieve(fetch, expected_rows, **kwrags):
    retries = kwrags.get('retries', 10)
    delay = kwrags.get('delay', 6)

    for i in range(1, retries):
        data = fetch()

        if len(data) == expected_rows:
            return data, None

        time.sleep(delay)

    return None, 'Reached maximum number of retries'
