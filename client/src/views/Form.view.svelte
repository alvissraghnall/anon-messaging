<script>
  import Footer from "../components/Footer.svelte";
  import FormCard from "../components/FormCard.svelte";
</script>

<div class="wrap-body">
  <FormCard>
    <slot></slot>
  </FormCard>
</div>
<Footer />

<style>
  .wrap-body {
    background-image: linear-gradient( 135deg, #92FFC0 10%, #002661 100%);
  }
</style>
