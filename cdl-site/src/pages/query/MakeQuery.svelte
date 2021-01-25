<script lang="ts">
  import { get } from "svelte/store";
  import { route, replaceRoute } from "../../route";
  import type { QueryRoute, QueryType } from "../../route";
  import { derived } from "svelte/store";
  import { getLoaded, validUuid } from "../../utils";
  import type { QueryResult, RemoteData } from "../../models";
  import { loaded, loading, notLoaded } from "../../models";
  import { schemas } from "../../stores";
  import { loadSchemas } from "../../api";

  import RemoteContent from "../../components/RemoteContent.svelte";
  import { mockData } from "../../sample-data";

  export let setResults: (results: RemoteData<QueryResult>) => void;

  const queryBy = derived(route, ($r) => ($r as QueryRoute).by || "single");

  let objectId: string = "";
  let objectIds: string = "";
  let schemaId: string = "";

  let objectIdError: string = "";
  let objectIdsError: string = "";

  if (get(schemas).status === "not-loaded") {
    loadSchemas();
  }

  function resetResults() {
    setResults(notLoaded);
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
    var errorsFound = false;
    var data: QueryResult | null = null;

    const by = get(queryBy);
    if (by === "single") {
      if (!validUuid(objectId)) {
        objectIdError = "Must provide a valid UUID";
        errorsFound = true;
      } else {
        const value = mockData.find(
          (d) => d.schemaId === schemaId && d.objectId === objectId
        );
        if (value) {
          data = new Map([[value.objectId, value.data]]);
        } else {
          data = new Map();
        }
      }
    } else if (by === "multiple") {
      const ids = objectIds.trim().split(/[ \r\n\t]+/);
      if (!ids.every(validUuid)) {
        objectIdsError = "Must provide valid UUID's";
        errorsFound = true;
      } else {
        data = new Map(
          mockData
            .filter((d) => d.schemaId === schemaId && ids.includes(d.objectId))
            .map((d) => [d.objectId, d.data])
        );
      }
    } else {
      data = new Map(
        mockData
          .filter((d) => d.schemaId === schemaId)
          .map((d) => [d.objectId, d.data])
      );
    }

    if (!errorsFound) {
      console.log(data);

      setResults(loading);
      setTimeout(() => {
        setResults(loaded(data));
      }, 1000);
    }
  }
</script>

<style>
  .run-query-button {
    margin-top: 10px;
  }
</style>

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
      <RemoteContent data={$schemas}>
        <select required bind:value={schemaId}>
          <option value="">Select a Schema</option>
          {#each getLoaded($schemas) || [] as schema}
            <option value={schema.id}>{schema.name}</option>
          {/each}
        </select>
        <select slot="loading" disabled>
          <option>loading...</option>
        </select>
      </RemoteContent>
    </label>
  </div>
  {#if $queryBy === 'single'}
    <div class="form-control">
      <label>
        Object ID
        <input
          class={objectIdError ? 'error' : ''}
          type="text"
          bind:value={objectId} />
      </label>
    </div>
    {#if objectIdError}
      <p class="validation-error">{objectIdError}</p>
    {/if}
  {:else if $queryBy === 'multiple'}
    <div class="form-control">
      <label>
        Object ID's
        <textarea
          class={objectIdsError ? 'invalid' : ''}
          bind:value={objectIds} />
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
