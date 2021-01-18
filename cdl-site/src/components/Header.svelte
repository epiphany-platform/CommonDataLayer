<script lang="ts">
  import { route } from "../route";
  import { darkMode } from "../stores";
  import Link from "./Link.svelte";

  let menuOpen: boolean = false;

  route.subscribe(() => {
    menuOpen = false;
  });
</script>

<style>
  nav {
    background: #346cc1;
  }

  nav.dark-mode {
    background: #9156d3;
  }

  .desktop-link {
    color: #fafafa;
  }

  .mobile-link {
    color: #424242;
  }

  ul.mobile-menu {
    position: absolute;
    top: 15px;
    right: 30px;
    z-index: 10;
  }

  ul.mobile-menu * {
    color: #424242;
  }

  .mobile-menu-toggle {
    color: #fafafa;
  }

  div.mobile-menu-background {
    position: fixed;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 9;
    background: rgba(0, 0, 0, 0.25);
  }
</style>

<nav class={`${$darkMode ? 'dark-mode' : ''}`}>
  <div class="nav-container">
    <div class="display-lg-up">
      <div class="nav-logo">
        <Link to={{ page: 'home' }}>Common Data Layer</Link>
      </div>
    </div>
    <div class="display-lg-down">
      <div class="nav-logo">
        <Link to={{ page: 'home' }}>CDL</Link>
      </div>
    </div>
    <ul class="nav-links">
      <li>
        <Link to={{ page: 'insert' }}>
          <span class="desktop-link">Insert</span>
        </Link>
      </li>
      <li>
        <Link to={{ page: 'query' }}>
          <span class="desktop-link">Query</span>
        </Link>
      </li>
      <li>
        <Link
          className={$route?.page === 'schemas' ? 'active' : ''}
          to={{ page: 'schemas' }}>
          <span class="desktop-link">Schemas</span>
        </Link>
      </li>
      <li>
        <Link to={{ page: 'settings' }}>
          <span class="desktop-link">Settings</span>
        </Link>
      </li>
    </ul>
    <span class="mobile-menu-toggle" on:click={() => (menuOpen = !menuOpen)} />
    {#if menuOpen}
      <div class="mobile-menu-background" on:click={() => (menuOpen = false)} />
    {/if}
    <ul
      class="mobile-menu menu"
      style={`display: ${menuOpen ? 'block' : 'none'};`}>
      <li>
        <Link to={{ page: 'insert' }}>
          <span class="mobile-link">Insert</span>
        </Link>
      </li>
      <li>
        <Link to={{ page: 'query' }}>
          <span class="mobile-link">Query</span>
        </Link>
      </li>
      <li>
        <Link
          className={$route?.page === 'schemas' ? 'active' : ''}
          to={{ page: 'schemas' }}>
          <span class="mobile-link">Schemas</span>
        </Link>
      </li>
      <li>
        <Link to={{ page: 'settings' }}>
          <span class="mobile-link">Settings</span>
        </Link>
      </li>
    </ul>
  </div>
</nav>
