<script>
  import { Router } from 'svelte-router-spa';
  import { routes } from './routes';
  import "./index.css";
</script>

<Router {routes} />

<style global lang="postcss">
  @tailwind base;
  @tailwind components;
  @tailwind utilities;
  @import "https://fonts.googleapis.com/css?family=Open+Sans:400,600,700|Roboto:300,300i,400,500,700";
  
  body {
    line-height: 24px;
    font-size: 14px;
    font-family: Roboto,sans-serif;
    font-weight: 400;
    color: #777;
  }

</style>
