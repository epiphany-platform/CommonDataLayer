<script lang="ts">
  import { get, derived } from "svelte/store";
  import { route, replaceRoute } from "../../route";
  import type { SchemasRoute } from "../../route";
  import type { AllSchemasQuery } from "../../generated/graphql";

  export let schemas: AllSchemasQuery["schemas"];

  const schemaId = derived(route, ($r) => ($r as SchemasRoute).id);
  const version = derived(route, ($r) => ($r as SchemasRoute).version);
  const schemaName = derived(
    schemaId,
    ($id) => (schemas || []).find((s) => s.id === $id)?.name
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

<style>
  ul.breadcrumbs {
    justify-content: center;
    font-size: 16px;
  }
</style>
