<script lang="ts">
  import { get, derived } from "svelte/store";
  import { schemas } from "../../stores";
  import { getLoaded } from "../../utils";
  import { route, replaceRoute } from "../../route";
  import type { SchemasRoute } from "../../route";

  import Link from "../../components/Link.svelte";

  export let fullWidth: boolean = false;

  const allSchemas = derived(schemas, ($schemas) => getLoaded($schemas) || []);
  const selectedId = derived(route, ($r) => ($r as SchemasRoute).id);
  const nameQuery = derived(route, ($r) => ($r as SchemasRoute).query);

  const visibleSchemas = derived(
    [allSchemas, nameQuery],
    ([$schemas, $query]) => {
      const lowerQuery = ($query || "").toLowerCase();
      return $schemas.filter((schema) =>
        schema.name.toLowerCase().includes(lowerQuery)
      );
    }
  );

  function setQuery(query: string) {
    replaceRoute({ ...(get(route) as SchemasRoute), query });
  }

  function selectSchema(schemaId: string) {
    replaceRoute({
      ...(get(route) as SchemasRoute),
      id: schemaId,
      version: undefined,
    });
  }
</script>

<style>
  .schema-name-query {
    padding-top: 10px;
    padding-bottom: 20px;
    border-bottom: 1px solid #e0e0e0;
    margin-bottom: 10px;
  }

  .add-schema-button {
    margin-top: 5px;
  }
</style>

<div class={`sidebar ${fullWidth ? '' : 'sidebar-left align-right'}`}>
  <h3 class="sidebar-category">Your Schemas</h3>
  <div class="schema-name-query">
    <input
      type="text"
      placeholder="Employee"
      value={$nameQuery || ''}
      on:input={(event) => setQuery(event.currentTarget.value)} />
  </div>
  <ul class="sidebar-links">
    {#if $visibleSchemas.length}
      {#each $visibleSchemas as schema}
        <li>
          <a
            title={schema.id}
            class={schema.id === $selectedId ? 'active' : ''}
            on:click={() => selectSchema(schema.id)}>
            {schema.name}
          </a>
        </li>
      {/each}
    {:else}
      <li><i>No schemas match the given query.</i></li>
    {/if}
    <li class="add-schema-button">
      <Link to={{ page: 'schemas', creating: true }}>
        <button>New Schema</button>
      </Link>
    </li>
  </ul>
</div>
