<script>
  import Footer from "../components/Footer.svelte";
  import FormCard from "../components/FormCard.svelte";
</script>

<div class="wrap-body">
  <FormCard>
    <div class="page-logo">
      <img src="assets/logo-icon.png" alt="Piree Logo">
    </div>
    <slot></slot>
  </FormCard>
</div>
<Footer />

<style>
  .wrap-body {
    background-image: linear-gradient( 135deg, #92FFC0 10%, #002661 100%);
  }
  
  .page-logo {
    display: block;
    font-family: Poppins-Bold;
    font-size: 39px;
    color: #fff;
    line-height: 1.2;
    text-align: center;
  }
  
  img {
    padding-bottom: 45px;
    vertical-align: middle;
    border-style: none;
  }
</style>
