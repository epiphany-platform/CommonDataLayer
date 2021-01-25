<script lang="ts">
  import { get, derived } from "svelte/store";
  import { schemas } from "../../stores";
  import { getLoaded } from "../../utils";
  import { loadSchemas } from "../../api";
  import { route } from "../../route";
  import type { SchemasRoute } from "../../route";

  import Overview from "./Overview.svelte";
  import Sidebar from "./Sidebar.svelte";
  import Link from "../../components/Link.svelte";
  import RemoteContent from "../../components/RemoteContent.svelte";

  const schemaId = derived(route, ($r) => ($r as SchemasRoute).id);
  const version = derived(route, ($r) => ($r as SchemasRoute).version);
  const creating = derived(route, ($r) => ($r as SchemasRoute).creating);

  if (get(schemas).status === "not-loaded") {
    loadSchemas();
  }

  const schema = derived([schemas, schemaId], ([$schemas, $schemaId]) =>
    (getLoaded($schemas) || []).find((schema) => schema.id === $schemaId)
  );
</script>

<style>
  .no-schema-selected {
    margin: auto;
    height: 100%;
    vertical-align: middle;
  }
</style>

<RemoteContent data={$schemas}>
  <div slot="loading" class="container container-small">
    <div class="row">
      <div class="col align-center">
        <h2>Your Schemas (are loading...)</h2>
        <div class="progress-bar striped animated">
          <span class="progress-bar-green" style="width: 100%;" />
        </div>
      </div>
    </div>
  </div>

  {#if getLoaded($schemas)?.length === 0}
    <div class="container container-small">
      <div class="row align-center">
        <div class="col align-center">
          <h2>Schemas</h2>
          <p>You have no schemas.</p>
          <p>
            <Link to={{ page: 'schemas', creating: true }}>
              <button>Create a Schema</button>
            </Link>
          </p>
        </div>
      </div>
    </div>
  {:else}
    <div class="container">
      <section>
        <div class="align-center">
          <h2>Schemas</h2>
        </div>
        <div class="display-md-down">
          {#if $schema}
            <Overview
              showBreadcrumbs={true}
              schema={$schema}
              version={$version} />
          {:else}
            <Sidebar fullWidth={true} />
          {/if}
        </div>
        <div class="display-md-up">
          <div class="row">
            <div class="col-sm-3">
              <Sidebar />
            </div>
            <div class="col-sm-9">
              {#if $schema}
                <Overview schema={$schema} version={$version} />
              {:else}
                <section class="no-schema-selected">
                  <p class="align-center">
                    Please select a schema from the left.
                  </p>
                </section>
              {/if}
            </div>
          </div>
        </div>
      </section>
    </div>
  {/if}
</RemoteContent>
