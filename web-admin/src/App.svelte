<script lang="ts">
  import Header from "./components/Header.svelte";
  import { route } from "./route";
  import { apiUrl, darkMode } from "./stores";

  import Home from "./pages/Home.svelte";
  import Insert from "./pages/insert/Insert.svelte";
  import Query from "./pages/query/Query.svelte";
  import Settings from "./pages/Settings.svelte";
  import Schemas from "./pages/schemas/Schemas.svelte";
  import NotFound from "./pages/NotFound.svelte";

  import { ApolloClient, InMemoryCache } from "@apollo/client";
  import { get } from "svelte/store";
  import { setClient } from "svelte-apollo";
  import { onDestroy } from "svelte";

  const apolloClient = new ApolloClient({
    cache: new InMemoryCache(),
    uri: get(apiUrl),
  });

  setClient(apolloClient);

  const unsubscribe = apiUrl.subscribe((uri) => {
    const apolloClient = new ApolloClient({
      cache: new InMemoryCache(),
      uri,
    });

    setClient(apolloClient);
  });

  onDestroy(() => unsubscribe());
</script>

<main>
  <div class={`${$darkMode ? "dark-mode" : ""}`}>
    <Header />
    {#if $route === null}
      <NotFound />
    {:else if $route.page === "home"}
      <Home />
    {:else if $route.page === "insert"}
      <Insert />
    {:else if $route.page === "query"}
      <Query />
    {:else if $route.page === "settings"}
      <Settings />
    {:else}
      <Schemas />
    {/if}
  </div>
</main>

<style>
  div.dark-mode {
    background: #424242;
    flex-direction: column;
    min-height: 100vh;
  }
</style>
