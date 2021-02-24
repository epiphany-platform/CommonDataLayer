<script lang="ts">
  import { get } from "svelte/store";
  import { route } from "../../route";
  import type { SchemasRoute } from "../../route";
  import type { Schema } from "../../models";

  import Breadcrumbs from "./Breadcrumbs.svelte";
  import Link from "../../components/Link.svelte";

  export let showBreadcrumbs: boolean = false;
  export let schema: Schema;
  export let version: string | null;

  $: versionDefinition = schema.versions.find((v) => v.version === version)
    ?.definition;
  $: versionPretty = versionDefinition
    ? JSON.stringify(JSON.parse(versionDefinition), null, 4)
    : "";

  function versionLink(version: string): SchemasRoute {
    return { ...(get(route) as SchemasRoute), version };
  }
</script>

<style>
  .breadcrumb-container {
    width: 100%;
  }

  .version {
    white-space: pre-wrap;
    text-align: left;
  }
</style>

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
          <td><b>Topic</b></td>
          <td>{schema.topic}</td>
        </tr>
        <tr>
          <td><b>Query Address</b></td>
          <td>{schema.queryAddress}</td>
        </tr>
        <tr>
          <td><b>Type</b></td>
          <td>{schema.schemaType}</td>
        </tr>
      </tbody>
    </table>
  </section>
  <section>
    <div class="row">
      <div class="col-sm-6">
        {#if schema.versions.length}
          <h5>Versions</h5>
          <ol>
            {#each schema.versions as version}
              <li>
                <Link replace={true} to={versionLink(version.version)}>
                  {version.version}
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
