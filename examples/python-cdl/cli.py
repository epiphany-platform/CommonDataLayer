import os
import re
import sys
import uuid
import cdl

from PyInquirer import prompt

LAST_SCHEMA_FILENAME = '.last_schema'

DEFAULT_INSERT_MESSAGE = """{{
    "schemaId": "{0}",
    "objectId": "{1}",
    "data": {{ "success": true }}
}}"""


def get_action():
    questions = [{
        'type': 'list',
        'name': 'action',
        'message': 'What do you want to do',
        'choices': ['Create new schema', 'Insert data into CDL', 'Retrieve data from CDL']
    }]

    return prompt(questions)


def get_last_schema_id():
    if not os.path.exists(LAST_SCHEMA_FILENAME):
        return None
    with open(LAST_SCHEMA_FILENAME) as f:
        guid = f.read()
        if re.match("^[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}$", guid):
            return guid
        else:
            return None


def random_uuid():
    return uuid.uuid1()


def insert_data():
    questions = [
        {
            'type': 'input',
            'name': 'kafka_brokers',
            'message': 'Kafka brokers',
        },
        {
            'type': 'input',
            'name': 'kafka_topic',
            'message': 'Kafka topic',
        },
        {
            'type': 'editor',
            'name': 'payload',
            'message': 'Message contents',
            'default': DEFAULT_INSERT_MESSAGE.format(get_last_schema_id() or random_uuid(), random_uuid()),
            'eargs': {
                'editor': 'vi',
                'ext': 'json',
            }
        }
    ]

    insert = prompt(questions)

    cdl.kafka_insert_data(insert['kafka_brokers'], insert['kafka_topic'], insert['payload'])


def retrieve_data():
    questions = [
        {
            'type': 'input',
            'name': 'rest_addr',
            'message': 'Query router endpoint',
            'default': 'http://',
        },
        {
            'type': 'input',
            'name': 'schema_id',
            'message': 'Schema ID',
            'default': get_last_schema_id() or '',
        },
        {
            'type': 'input',
            'name': 'object_id',
            'message': 'Message Identifier',
        }
    ]

    query = prompt(questions)

    object = cdl.query_get_single(query['rest_addr'], query['schema_id'], query['object_id'])

    print(object.json())


def create_schema():
    questions = [
        {
            'type': 'input',
            'name': 'schema_url',
            'message': 'Schema Registry address'
        },
        {
            'type': 'input',
            'name': 'schema_name',
            'message': 'Name of schema to insert',
        },
        {
            'type': 'input',
            'name': 'schema_topic',
            'default': 'cdl.document.input',
            'message': 'Kafka topic for data-router to route messages to',
        },
        {
            'type': 'list',
            'name': 'schema_type',
            'choices': ['DocumentStorage', 'Timeseries'],
            'message': 'Type of repository',
        },
        {
            'type': 'input',
            'name': 'schema_query',
            'default': 'http://postgres_query:50102',
            'message': 'Query-service address for query-router to submit queries',
        },
        {
            'type': 'editor',
            'name': 'schema_body',
            'message': 'Schema body',
            'default': '{}',
            'eargs': {
                'editor': 'vi',
                'ext': 'json',
            }
        }
    ]

    create = prompt(questions)

    schema_id = cdl.registry_create_schema(create['schema_url'], create['schema_name'], create['schema_topic'],
                                           create['schema_query'], create['schema_body'], create['schema_type'])

    print(f"Schema was assigned id: {schema_id}")

    with open(LAST_SCHEMA_FILENAME, 'w') as f:
        f.write(schema_id)


if __name__ == "__main__":
    answers = get_action()
    if answers['action'][0] == 'I':
        insert_data()
    elif answers['action'][0] == 'R':
        retrieve_data()
    elif answers['action'][0] == 'C':
        create_schema()
    else:
        print("Invalid option!", file=sys.stderr)
