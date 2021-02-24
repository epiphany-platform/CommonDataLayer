import { get } from "svelte/store";
import { schemas, apiUrl } from "./stores";
import type { RemoteData } from "./models";
import { loading, loaded } from "./models";
import { allSchemas, mockData } from "./sample-data";

export async function queryApi<T>(query: string, variables?: Object): Promise<RemoteData<T>> {
  try {
    const response = await fetch(get(apiUrl), {
      method: "post",
      mode: "cors",
      headers: {
        "Content-Type": "application/json",
        "Accept": "application/json"
      },
      body: JSON.stringify({ query, variables })
    });
    const data = await response.json();

    if (data?.errors?.length) {
      return { status: "error", error: data.errors[0].message };
    } else {
      return { status: "loaded", data: data.data };
    }
  } catch (exception) {
    return { status: "error", error: exception.toString() };
  }
}

export const GET_SCHEMAS = ``;

export function loadSchemas() {
  schemas.set(loading);
  setTimeout(() => {
    schemas.set(loaded(allSchemas));
  }, 1000);
};
