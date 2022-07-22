<script>
  import { Navigate } from "svelte-router-spa";
</script>

<section>
  <div class="content-box">
    <h2>
      Send secret anonymous messages online
    </h2>

    <p>
      Piree is an interactive messaging app with a dare game. Create your Profile Link and Send it to all your contacts to check what do your friends think about you. With the help of Piree, you can send, recieve, ans reply to anonymous compliments easily for free! Great, innit?
    </p>
    
    <button class="relative inline-flex items-center justify-center p-0.5 mb-2 mr-2 overflow-hidden text-sm font-medium text-gray-900 rounded-lg group bg-gradient-to-br from-green-400 to-blue-600 group-hover:from-green-400 group-hover:to-blue-600 hover:text-white dark:text-white focus:ring-4 focus:outline-none focus:ring-green-200 dark:focus:ring-green-800">
      <Navigate to="register">
        <span class="relative px-5 py-2.5 transition-all ease-in duration-75 bg-white dark:bg-gray-900 rounded-md group-hover:bg-opacity-0">
          Get Started
        </span>
      </Navigate>
    </button>
  </div>
</section>

<style>
  section {
    background-image: url("assets/home-banner.jpg");
    background-repeat: no-repeat;
    background-attachment: scroll;
    background-position: center center;
    background-size: cover;
    min-height: 400px;
    text-align: left;
    
  }

  .content-box {
    margin: .22em;
  }

  h2 {
    font-size: 1.74em;
    margin-top: 1.5em;
    font-weight: 700;
    margin-bottom: .89em;
    font-family: 'Lucida Sans', 'Lucida Sans Regular', 'Lucida Grande', 'Lucida Sans Unicode', Geneva, Verdana, sans-serif;
  }
</style>
