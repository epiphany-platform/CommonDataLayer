<script lang="ts">
  import type { InsertMessage } from "../../models";

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
      {#each messages as message}
        <li><span>{message.objectId}</span></li>
      {/each}
    {:else}
      <li><i>No messages in transaction.</i></li>
    {/if}
    <li class="send-button">
      <button disabled={messages.length === 0} on:click={sendTransaction}>
        Send Transaction
      </button>
    </li>
  </ul>
</div>
