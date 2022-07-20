<script>
  import { Router } from 'svelte-router-spa';
  import { routes } from './routes';
  import "./tailwind.css";
</script>

<Router {routes} />

<style global lang="postcss">
  @tailwind base;
  @tailwind components;
  @tailwind utilities;
</style>
