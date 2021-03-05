<script lang="ts">
  import Header from "./components/Header.svelte";
  import { route } from "./route";
  import { darkMode } from "./stores";

  import Home from "./pages/Home.svelte";
  import Insert from "./pages/insert/Insert.svelte";
  import Query from "./pages/query/Query.svelte";
  import Settings from "./pages/Settings.svelte";
  import Schemas from "./pages/schemas/Schemas.svelte";
  import NotFound from "./pages/NotFound.svelte";
</script>

<style>
  div.dark-mode {
    background: #424242;
    flex-direction: column;
    min-height: 100vh;
  }
</style>

<main>
  <div class={`${$darkMode ? 'dark-mode' : ''}`}>
    <Header />
    {#if $route === null}
      <NotFound />
    {:else if $route.page === 'home'}
      <Home />
    {:else if $route.page === 'insert'}
      <Insert />
    {:else if $route.page === 'query'}
      <Query />
    {:else if $route.page === 'settings'}
      <Settings />
    {:else}
      <Schemas />
    {/if}
  </div>
</main>
