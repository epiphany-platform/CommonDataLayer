import { schemas } from "./stores";
import { loading, loaded } from "./models";
import { getClient } from "svelte-apollo";
import { gql } from "@apollo/client";

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
  schemas.set(
    loaded(
      r.data.schemas.map(s => ({
        name: s.name,
        id: s.id,
        topic: s.insertDestination,
        queryAddress: s.queryAddress,
        schemaType: s.type,
        versions: s.definitions.map((d) => ({
          version: d.version,
          definition: d.definition,
        })),
      })),
    ),
  );
}
