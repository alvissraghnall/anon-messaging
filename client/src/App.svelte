<script>
  import { Router } from 'svelte-router-spa';
  import { routes } from './routes'
</script>

<Router {routes} />

<style global lang="postcss">
  @tailwind base;
  @tailwind components;
  @tailwind utilities;
</style>
