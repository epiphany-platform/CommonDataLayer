<script lang="ts">
  import Link from "../components/Link.svelte";
</script>

<div class="container container-small">
  <div class="row">
    <div class="col align-center">
      <h2>Error 404</h2>
      <p>That page doesn't exist.</p>
      <Link to={{ page: 'home' }}><button>Back to Home</button></Link>
    </div>
  </div>
</div>
