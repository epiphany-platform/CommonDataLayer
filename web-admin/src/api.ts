import {graphqlClient, schemas} from "./stores";
import {loaded, loading, SchemaKind} from "./models";
import {get} from "svelte/store";

export async function loadSchemas() {
    schemas.set(loading);
    const resp = await get(graphqlClient).AllSchemas();

    schemas.set(
        loaded(resp.schemas.map(s => ({
            name: s.name,
            id: s.id,
            topic: s.insertDestination,
            queryAddress: s.queryAddress,
            schemaType: SchemaKind[s.type],
            versions: s.definitions.map((d) => ({
                version: d.version,
                definition: d.definition,
            })),
        })))
    );
}
