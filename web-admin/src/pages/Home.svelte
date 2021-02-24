<script lang="ts">
  import Link from "../components/Link.svelte";
</script>

<div class="container container-small">
  <div class="row">
    <div class="col align-center">
      <h2><u>The Common Data Layer</u></h2>
      <p>
        The Common Data Layer (CDL) is a data storage service. It is designed
        with performance, versatility, scalability, and ease-of-modification as
        key tenets of its design, among others.
      </p>
      <p><i>TODO: ADD A DIAGRAM HERE</i></p>
      <p>
        Get started by
        <Link to={{ page: 'schemas' }}><u>creating a schema</u></Link>
        for your data.
      </p>
      <p>
        You can learn more by visiting our
        <a href="https://github.com/epiphany-platform/CommonDataLayer">
          <u>GitHub page</u></a>.
      </p>
    </div>
  </div>
</div>
