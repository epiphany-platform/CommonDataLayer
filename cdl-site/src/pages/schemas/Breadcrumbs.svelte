<script lang="ts">
  import { get, derived } from "svelte/store";
  import { route, replaceRoute } from "../../route";
  import type { SchemasRoute } from "../../route";
  import { schemas } from "../../stores";
  import { getLoaded } from "../../utils";

  const schemaId = derived(route, ($r) => ($r as SchemasRoute).id);
  const version = derived(route, ($r) => ($r as SchemasRoute).version);
  const schemaName = derived(
    [schemaId, schemas],
    ([$id, $schemas]) =>
      getLoaded($schemas)?.find((schema) => schema.id === $id)?.name
  );

  function backToAllSchemas() {
    replaceRoute({
      ...(get(route) as SchemasRoute),
      id: undefined,
      version: undefined,
    });
  }

  function backToSchema() {
    replaceRoute({ ...(get(route) as SchemasRoute), version: undefined });
  }
</script>

<style>
  ul.breadcrumbs {
    justify-content: center;
    font-size: 16px;
  }
</style>

<ul class="breadcrumbs">
  <li><a on:click={backToAllSchemas}>All Schemas</a></li>
  {#if $schemaId}
    {#if !$version}
      <li>{$schemaName}</li>
    {:else}
      <li><a on:click={backToSchema}>{$schemaName}</a></li>
      <li>{$version}</li>
    {/if}
  {/if}
</ul>
