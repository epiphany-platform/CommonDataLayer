<script lang="ts">
  import { getLoaded } from "../../utils";
  import type { CdlObject } from "../../generated/graphql";

  import MakeQuery from "./MakeQuery.svelte";
  import LoadingBar from "../../components/LoadingBar.svelte";

  let results: CdlObject[] | null = null;
  let loading = false;

  $: resultsPretty = JSON.stringify(
    Array.from(getLoaded(results) || []).reduce((obj, [key, value]) => {
      obj[key] = value;
      return obj;
    }, {}),
    null,
    4
  );

  function setResults(res: Promise<CdlObject[]> | null) {
    if (!res) {
      results = null;
      loading = false;
    } else {
      loading = true;
      res
        .then((data) => {
          results = data;
        })
        .finally(() => {
          loading = false;
        });
    }
  }
</script>

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
          {#if !results}
            <p>Make a query to see data.</p>
          {:else if loading}
            <LoadingBar />
          {:else}
            <pre class="data">{resultsPretty}</pre>
          {/if}
        </section>
      </div>
    </div>
  </section>
</div>

<style>
  .data {
    white-space: pre-wrap;
    text-align: left;
  }
</style>
