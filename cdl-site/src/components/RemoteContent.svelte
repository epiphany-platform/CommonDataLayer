<script lang="ts">
  import type { RemoteData } from "../models";
  import Error from "./Error.svelte";
  import LoadingBar from "./LoadingBar.svelte";

  export let data: RemoteData<any>;
</script>

{#if data.status === 'loaded'}
  <slot />
{:else if data.status === 'not-loaded'}
  <slot name="not-loaded" />
{:else if data.status === 'loading'}
  <slot name="loading">
    <LoadingBar />
  </slot>
{:else}
  <slot name="error">
    <Error error={data.status === 'error' ? data.error : ''} />
  </slot>
{/if}
