<script lang="ts">
  import { get } from "svelte/store";
  import { route } from "../../route";
  import type { SchemasRoute } from "../../route";

  import Breadcrumbs from "./Breadcrumbs.svelte";
  import Link from "../../components/Link.svelte";
  import type { AllSchemasQuery } from "../../generated/graphql";

  export let showBreadcrumbs: boolean = false;
  export let schema: AllSchemasQuery["schemas"][0];
  export let version: string | null;

  $: versionDefinition = schema.definitions.find((d) => d.version === version)
    ?.definition;
  $: versionPretty = versionDefinition
    ? JSON.stringify(versionDefinition, null, 4)
    : "";

  function versionLink(version: string): SchemasRoute {
    return { ...(get(route) as SchemasRoute), version };
  }
</script>

<div class="col-8 align-center">
  <section>
    {#if showBreadcrumbs}
      <div class="breadcrumb-container">
        <Breadcrumbs />
      </div>
    {/if}

    <table>
      <tbody>
        <tr>
          <td><b>Name</b></td>
          <td>{schema.name}</td>
        </tr>
        <tr>
          <td><b>ID</b></td>
          <td>{schema.id}</td>
        </tr>
        <tr>
          <td><b>Insert Destination</b></td>
          <td>{schema.insertDestination}</td>
        </tr>
        <tr>
          <td><b>Query Address</b></td>
          <td>{schema.queryAddress}</td>
        </tr>
        <tr>
          <td><b>Type</b></td>
          <td>{schema.type}</td>
        </tr>
      </tbody>
    </table>
  </section>
  <section>
    <div class="row">
      <div class="col-sm-6">
        {#if schema.definitions.length}
          <h5>Versions</h5>
          <ol>
            {#each schema.definitions as definition}
              <li>
                <Link replace={true} to={versionLink(definition.version)}>
                  {definition.version}
                </Link>
              </li>
            {/each}
          </ol>
        {/if}
      </div>
      <div class="col-sm-6">
        {#if version}
          <pre class="version">
            {versionPretty}
          </pre>
        {:else}
          <p>Please select a version.</p>
        {/if}
      </div>
    </div>
  </section>
</div>

<style>
  .breadcrumb-container {
    width: 100%;
  }

  .version {
    white-space: pre-wrap;
    text-align: left;
  }
</style>
