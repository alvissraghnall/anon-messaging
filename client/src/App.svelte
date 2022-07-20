<script>
  import { Router } from 'svelte-router-spa';
  import { routes } from './routes'
</script>

<Router {routes} />

<style lang="postcss" global>
  @tailwind base;
  @tailwind components;
  @tailwind utilities;
</style>
