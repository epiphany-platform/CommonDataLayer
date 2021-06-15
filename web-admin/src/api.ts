import { get } from "svelte/store";
import { schemas, apiUrl } from "./stores";
import type { RemoteData, Schema } from "./models";
import { loading, loaded } from "./models";
import { getClient } from "svelte-apollo";
import { gql } from "@apollo/client";

export async function queryApi<T>(
  query: string,
  variables?: Object
): Promise<RemoteData<T>> {
  try {
    const response = await fetch(get(apiUrl), {
      method: "post",
      mode: "cors",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({ query, variables }),
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

export async function loadSchemas() {
  schemas.set(loading);
  const client = getClient();
  const r = await client.query({
    query: gql`
      query AllSchemas {
        schemas {
          id
          name
          insertDestination
          queryAddress
          type
          definitions {
            version
            definition
          }
        }
      }
    `,
  });
  console.log(r);
  schemas.set(
    loaded(
      r.data.schemas.map(function (s) {
        return {
          name: s.name,
          id: s.id,
          topic: s.insertDestination,
          queryAddress: s.queryAddress,
          schemaType: s.type,
          versions: s.definitions.map(function (d) {
            return {
              version: d.version,
              definition: d.definition,
            };
          }),
        };
      })
    )
  );
}
