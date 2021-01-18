import type { Schema, InsertMessage } from "./models";

export const allSchemas: Schema[] = [
  {
    name: "Work Order",
    id: "9262ec4f-3470-4f9a-ab68-2a20aa1988a1",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "DocumentStorage",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Item",
    id: "29ff99d5-10ec-4b00-be71-a16380fad3a9",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "DocumentStorage",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 3",
    id: "677103e5-06c3-4e7f-9268-c581cade1940",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "Timeseries",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 4",
    id: "41cb0f27-ea34-4e9f-858f-88af982c9441",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "DocumentStorage",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 5",
    id: "bb5bd9bc-863f-4801-9cc6-2e154fe98d31",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "Timeseries",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 6",
    id: "a95e093c-200f-4d36-802b-5eb5320680ee",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "Timeseries",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 7",
    id: "7318a478-92e9-4e4f-9253-fc11717e91ef",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "Timeseries",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
  {
    name: "Schema 8",
    id: "5464cc54-5930-4380-9aaf-407016738098",
    topic: "document-insert-topic",
    queryAddress: "https://query-address.com/",
    schemaType: "DocumentStorage",
    versions: [
      {
        version: "1.0.0",
        definition: "{ \"a\": 123 }"
      }
    ]
  },
];

export const mockData: InsertMessage[] = [
  {
    schemaId: "9262ec4f-3470-4f9a-ab68-2a20aa1988a1",
    objectId: "cd4a2d63-0a2c-4f0f-801f-0c73deff277d",
    data: { some: "data 1" }
  },
  {
    schemaId: "9262ec4f-3470-4f9a-ab68-2a20aa1988a1",
    objectId: "e2231d08-63d2-403f-a4d1-09d69213adb0",
    data: { some: "data 2" }
  },
  {
    schemaId: "9262ec4f-3470-4f9a-ab68-2a20aa1988a1",
    objectId: "243f7c62-0cbe-4b81-809f-baba96daf57c",
    data: { some: "data 3" }
  },
  {
    schemaId: "9262ec4f-3470-4f9a-ab68-2a20aa1988a1",
    objectId: "431adb0f-3cc6-4022-96d4-62137fbc108b",
    data: { some: "data 4" }
  },
  {
    schemaId: "29ff99d5-10ec-4b00-be71-a16380fad3a9",
    objectId: "581cd4b8-7043-40cf-a524-e1cd0652605d",
    data: { some: "data 5" }
  },
  {
    schemaId: "29ff99d5-10ec-4b00-be71-a16380fad3a9",
    objectId: "a96d4792-a68f-4262-b0b5-aa4341f80ca6",
    data: { some: "data 6" }
  },
  {
    schemaId: "29ff99d5-10ec-4b00-be71-a16380fad3a9",
    objectId: "946afff9-126f-4a7c-bf26-7869021efda5",
    data: { some: "data 7" }
  },
];
