<script lang="ts">
  import type { QueryResult, RemoteData } from "../../models";
  import { notLoaded } from "../../models";
  import { getLoaded } from "../../utils";

  import RemoteContent from "../../components/RemoteContent.svelte";
  import MakeQuery from "./MakeQuery.svelte";

  let results: RemoteData<QueryResult> = notLoaded;
  $: resultsPretty = JSON.stringify(
    Array.from(getLoaded(results) || []).reduce((obj, [key, value]) => {
      obj[key] = value;
      return obj;
    }, {}),
    null,
    4
  );

  function setResults(res: RemoteData<QueryResult>) {
    results = res;
  }
</script>

<style>
  .data {
    white-space: pre-wrap;
    text-align: left;
  }
</style>

<div class="container">
  <div class="row">
    <div class="col align-center">
      <h2>Query Data</h2>
    </div>
  </div>
  <section>
    <div class="row">
      <div class="col-sm-4">
        <MakeQuery {setResults} />
      </div>
      <div class="col-sm-8 align-center">
        <section>
          <h4>Results</h4>
          <RemoteContent data={results}>
            <pre class="data">{resultsPretty}</pre>
            <p slot="not-loaded">Make a query to see data.</p>
          </RemoteContent>
        </section>
      </div>
    </div>
  </section>
</div>
