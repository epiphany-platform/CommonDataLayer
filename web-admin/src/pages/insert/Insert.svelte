<script lang="ts">
  import type { InsertMessage } from "../../models";
  import MessageList from "./MessageList.svelte";
  import MessageForm from "./MessageForm.svelte";

  let transaction: InsertMessage[] = [];

  function sendTransaction() {
    alert("Insertion is not supported by the GraphQL API at the moment.");
  }
</script>

<style>
  .form-container {
    margin-left: auto;
    margin-right: auto;
  }
</style>

<div class="container">
  <div class="row">
    <div class="col align-center">
      <h2>Insert Data</h2>
    </div>
  </div>
  <section>
    <div class="row">
      <div class="col-sm-3 align-right">
        <MessageList messages={transaction} {sendTransaction} />
      </div>
      <div class="form-container col-sm-8 col-sm-offset-1">
        <MessageForm
          addMessage={(message) => (transaction = [...transaction, message])} />
      </div>
    </div>
  </section>
</div>
