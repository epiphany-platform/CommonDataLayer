<script lang="ts">
  import { get } from "svelte/store";
  import type { InsertMessage } from "../../models";
  import { getLoaded, validUuid } from "../../utils";
  import { schemas } from "../../stores";
  import { loadSchemas } from "../../api";
  import RemoteContent from "../../components/RemoteContent.svelte";
  import { v4 as uuidv4 } from "uuid";

  export let addMessage: (message: InsertMessage) => void;

  let objectId: string = "";
  let schemaId: string = "";
  let data: string = "";

  let objectIdError: string = "";
  let dataError: string = "";

  if (get(schemas).status === "not-loaded") {
    loadSchemas();
  }

  function submit() {
    objectIdError = "";
    dataError = "";
    var errorsFound = false;

    if (objectId.length > 0 && !validUuid(objectId)) {
      objectIdError = "Invalid UUID";
      errorsFound = true;
    }

    var parsedData: Object | null = null;
    try {
      parsedData = JSON.parse(data);
    } catch (exception) {
      dataError = "Invalid JSON";
      errorsFound = true;
    }

    if (!errorsFound) {
      addMessage({
        objectId: objectId || uuidv4(),
        schemaId,
        data: parsedData,
      });

      objectId = "";
      schemaId = "";
      data = "";
    }
  }
</script>

<form on:submit|preventDefault={submit}>
  <div class="form-control">
    <label>Object ID
      <input
        type="text"
        bind:value={objectId}
        class={objectIdError ? 'invalid' : ''}
        placeholder="Leave empty for a random ID" />
      {#if objectIdError}
        <p class="validation-error">{objectIdError}</p>
      {/if}
    </label>
  </div>
  <div class="form-control">
    <label>Schema ID</label>
    <RemoteContent data={$schemas}>
      {#if getLoaded($schemas)}
        <select required bind:value={schemaId}>
          <option value="">Select a Schema</option>
          {#each getLoaded($schemas) || [] as schema}
            <option value={schema.id}>{schema.name}</option>
          {/each}
        </select>
      {/if}

      <select disabled slot="loading">
        <option>Loading...</option>
      </select>
    </RemoteContent>
  </div>
  <div class="form-control">
    <label>Payload
      <textarea
        required
        bind:value={data}
        class={dataError ? 'invalid' : ''}
        placeholder="JSON content goes here..." />
    </label>
    {#if dataError}
      <p class="validation-error">{dataError}</p>
    {/if}
  </div>
  <button type="submit">Add Message to Transaction</button>
</form>
