export interface InsertMessage {
  objectId: string;
  schemaId: string;
  data: Object;
}

export type QueryResult = Map<string, Object>;

export interface Schema {
  id: string;
  name: string;
  topic: string;
  queryAddress: string;
  schemaType: SchemaType;
  versions: SchemaVersion[];
}

export interface SchemaVersion {
  version: string;
  definition: string;
}

export interface NewSchema {
  id: string;
  name: string;
  topic: string;
  queryAddress: string;
  definition: string;
  schemaType: SchemaType;
}

export type SchemaType = "DocumentStorage" | "Timeseries";

export type RemoteData<T> =
  | { status: "not-loaded" }
  | { status: "loading" }
  | { status: "loaded", data: T }
  | { status: "error", error: string };

export const notLoaded: { status: "not-loaded" } = { status: "not-loaded" };
export const loading: { status: "loading" } = { status: "loading" };
export const loaded = <T extends any>(data: T): RemoteData<T> => ({ status: "loaded", data });
export const loadingError = <T extends any>(error: string): RemoteData<T> => ({ status: "error", error });
