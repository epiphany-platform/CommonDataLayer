<script lang="ts">
  import type { InsertMessage } from "../../models";
  import { schemas } from "../../stores";
  import { getLoaded } from "../../utils";

  export let messages: InsertMessage[];
  export let sendTransaction: () => void;
</script>

<style>
  .send-button {
    margin-top: 10px;
  }
</style>

<div class="sidebar sidebar-left align-right">
  <h3 class="sidebar-category">Messages in Transaction</h3>
  <ul class="sidebar-links">
    {#if messages.length}
      {#each messages as message, index}
        <li>
          <span>
            <b>{index + 1}:</b>
            {JSON.stringify(message).length / 500}
            kb,
            {(getLoaded($schemas) || []).find((s) => s.id === message.schemaId)?.name || 'Unknown schema'}
          </span>
        </li>
      {/each}
    {:else}
      <li><i>No messages in transaction.</i></li>
    {/if}
    <li class="send-button">
      <button disabled={!messages.length} on:click={sendTransaction}>
        Send
      </button>
    </li>
  </ul>
</div>
