<script lang="ts">
  import MessageList from "./MessageList.svelte";
  import MessageForm from "./MessageForm.svelte";
  import { AllSchemas, InsertBatch } from "../../generated/graphql";
  import type { InputMessage } from "../../generated/graphql";

  let messages: InputMessage[] = [];
  let loading = false;

  $: schemas = AllSchemas({});

  function sendTransaction() {
    loading = true;

    InsertBatch({ variables: { messages } })
      .then(() => {
        alert(`Successfully inserted ${messages.length} rows.`);
        messages = [];
      })
      .catch(() => {
        alert("Failed to insert messages");
      })
      .finally(() => {
        loading = false;
      });
  }
</script>

<div class="container">
  <div class="row">
    <div class="col align-center">
      <h2>Insert Data</h2>
    </div>
  </div>
  <section>
    <div class="row">
      <div class="col-sm-3 align-right">
        <MessageList
          {messages}
          {sendTransaction}
          schemas={$schemas.data?.schemas || null}
        />
      </div>
      <div class="form-container col-sm-8 col-sm-offset-1">
        <MessageForm
          addMessage={(message) => (messages = [...messages, message])}
          schemas={$schemas.data?.schemas || null}
        />
      </div>
    </div>
  </section>
</div>

<style>
  .form-container {
    margin-left: auto;
    margin-right: auto;
  }
</style>
