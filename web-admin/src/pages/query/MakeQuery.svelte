<script lang="ts">
  import { get } from "svelte/store";
  import { route, replaceRoute } from "../../route";
  import type { QueryRoute, QueryType } from "../../route";
  import { derived } from "svelte/store";
  import { validUuid } from "../../utils";

  import {
    AsyncSingleObject,
    AsyncSchemaObjects,
    AsyncMultipleObjects,
    AllSchemas,
  } from "../../generated/graphql";
  import type { CdlObject } from "../../generated/graphql";

  export let setResults: (results: Promise<CdlObject[] | null>) => void;

  const queryBy = derived(route, ($r) => ($r as QueryRoute).by || "single");

  let objectId: string = "";
  let objectIds: string = "";
  let schemaId: string = "";

  let objectIdError: string = "";
  let objectIdsError: string = "";

  $: schemas = AllSchemas({});

  function resetResults() {
    setResults(null);
  }

  function setQueryType(event: { currentTarget: { value: string } }) {
    const by = event.currentTarget.value as QueryType;
    const $route = get(route);
    if ($route.page === "query" ? $route.by === by : false) return;

    replaceRoute({ page: "query", by });
    objectId = "";
    objectIds = "";

    objectIdError = "";
    objectIdsError = "";
  }

  function submit() {
    objectIdError = "";
    objectIdsError = "";

    const by = get(queryBy);
    if (by === "single") {
      if (!validUuid(objectId)) {
        objectIdError = "Must provide a valid UUID";
        return;
      }

      setResults(
        AsyncSingleObject({ variables: { objectId, schemaId } }).then((res) =>
          res.data?.object ? [res.data.object] : null
        )
      );
    } else if (by === "multiple") {
      const ids = objectIds.trim().split(/[ \r\n\t]+/);
      if (!ids.every(validUuid)) {
        objectIdsError = "Must provide valid UUID's";
        return;
      }

      setResults(
        AsyncMultipleObjects({ variables: { objectIds: ids, schemaId } }).then(
          (res) => res.data?.objects || null
        )
      );
    } else {
      setResults(
        AsyncSchemaObjects({ variables: { schemaId } }).then(
          (res) => res.data?.schemaObjects || null
        )
      );
    }
  }
</script>

<form on:submit|preventDefault={submit}>
  <div class="form-control">
    <label>
      Query Type
      <select value={$queryBy} on:blur={setQueryType} on:input={setQueryType}>
        <option value="single">Single Value</option>
        <option value="multiple">Multiple Values</option>
        <option value="schema">By Schema</option>
      </select>
    </label>
  </div>
  <div class="form-control">
    <label>
      Schema
      {#if $schemas.loading}
        <select disabled>
          <option>Loading...</option>
        </select>
      {:else}
        <select required bind:value={schemaId}>
          <option value="">Select a Schema</option>
          {#each $schemas.data?.schemas || [] as schema}
            <option value={schema.id}>{schema.name}</option>
          {/each}
        </select>
      {/if}
    </label>
  </div>
  {#if $queryBy === "single"}
    <div class="form-control">
      <label>
        Object ID
        <input
          class={objectIdError ? "error" : ""}
          type="text"
          bind:value={objectId}
        />
      </label>
    </div>
    {#if objectIdError}
      <p class="validation-error">{objectIdError}</p>
    {/if}
  {:else if $queryBy === "multiple"}
    <div class="form-control">
      <label>
        Object ID's
        <textarea
          class={objectIdsError ? "invalid" : ""}
          bind:value={objectIds}
        />
      </label>
    </div>
    {#if objectIdsError}
      <p class="validation-error">{objectIdsError}</p>
    {/if}
  {/if}
  <div class="align-right">
    <button type="button" on:click={resetResults}>Clear</button>
    <button class="run-query-button" type="submit">Run Query</button>
  </div>
</form>

<style>
  .run-query-button {
    margin-top: 10px;
  }
</style>
