<script lang="ts">
  import { darkMode, apiUrl } from "../stores";
</script>

<div class="container container-small">
  <div class="row">
    <div class="col align-center">
      <h2>Settings</h2>
      <section>
        <p>
          <button on:click={() => darkMode.update((m) => !m)}>
            Your colorscheme is
            {$darkMode ? 'Dark Mode' : 'Light Mode'}
          </button>
        </p>
        <div class="form-control">
          <label>
            Your CDL's GraphQL API URL
            <input
              type="url"
              value={$apiUrl}
              on:input={(event) => apiUrl.set(event.currentTarget.value)} />
          </label>
        </div>
      </section>
    </div>
  </div>
</div>
