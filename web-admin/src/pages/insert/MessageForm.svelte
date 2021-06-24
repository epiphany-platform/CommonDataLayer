<script lang="ts">
  import { validUuid } from "../../utils";
  import { v4 as uuidv4 } from "uuid";
  import type { AllSchemasQuery, InputMessage } from "../../generated/graphql";

  export let addMessage: (message: InputMessage) => void;
  export let schemas: AllSchemasQuery["schemas"] | null;

  let objectId: string = "";
  let schemaId: string = "";
  let data: string = "";

  let objectIdError: string = "";
  let dataError: string = "";

  function submit() {
    objectIdError = "";
    dataError = "";
    var errorsFound = false;

    if (objectId.length > 0 && !validUuid(objectId)) {
      objectIdError = "Invalid UUID";
      errorsFound = true;
    }

    try {
      JSON.parse(data);
    } catch (exception) {
      dataError = "Invalid JSON";
      errorsFound = true;
    }

    if (!errorsFound) {
      addMessage({
        objectId: objectId || uuidv4(),
        schemaId,
        payload: data,
      });

      objectId = "";
      schemaId = "";
      data = "";
    }
  }
</script>

<form on:submit|preventDefault={submit}>
  <div class="form-control">
    <label
      >Object ID
      <input
        type="text"
        bind:value={objectId}
        class={objectIdError ? "invalid" : ""}
        placeholder="Leave empty for a random ID"
      />
      {#if objectIdError}
        <p class="validation-error">{objectIdError}</p>
      {/if}
    </label>
  </div>
  <div class="form-control">
    <label>Schema ID</label>
    {#if !schemas}
      <select disabled>
        <option>Loading...</option>
      </select>
    {:else}
      <select required bind:value={schemaId}>
        <option value="">Select a Schema</option>
        {#each schemas as schema}
          <option value={schema.id}>{schema.name}</option>
        {/each}
      </select>
    {/if}
  </div>
  <div class="form-control">
    <label
      >Payload
      <textarea
        required
        bind:value={data}
        class={dataError ? "invalid" : ""}
        placeholder="JSON content goes here..."
      />
    </label>
    {#if dataError}
      <p class="validation-error">{dataError}</p>
    {/if}
  </div>
  <button type="submit">Add Message to Transaction</button>
</form>
