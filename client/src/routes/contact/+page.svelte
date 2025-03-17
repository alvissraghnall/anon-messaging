<script lang="ts">
  let name = '';
  let email = '';
  let message = '';
  let sending = false;
  let sent = false;
  let error = '';
  
  async function handleSubmit(ev: Event) {
    ev.preventDefault();
    if (!name || !email || !message) {
      error = 'Please fill out all fields';
      return;
    }
    
    sending = true;
    error = '';
    
    try {
      // const response = await fetch('/api/contact', {
      //   method: 'POST',
      //   body: JSON.stringify({ name, email, message }),
      //   headers: { 'Content-Type': 'application/json' }
      // });
      
      // if (!response.ok) throw new Error('Failed to send message');
      
      await new Promise(resolve => setTimeout(resolve, 10000));
      
      sent = true;
      name = '';
      email = '';
      message = '';
    } catch (e) {
      error = 'Failed to send message. Please try again.';
    } finally {
      sending = false;
    }
  }
</script>

<svelte:head>
  <title>Contact Us | Piree Anonymous Messaging Service</title>
  <meta name="description" content="Contact the team behind Piree anonymous messaging service">
</svelte:head>

<div class="min-h-screen transition-colors duration-200 bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
  <div class="container mx-auto px-4 py-16 max-w-4xl">
    
    <div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg overflow-hidden">
      <div class="px-6 py-8 md:p-10">
        <h1 class="text-3xl font-bold mb-2 text-center text-gray-900 dark:text-gray-100">Contact Us</h1>
        <p class="text-center mb-8 text-gray-600 dark:text-gray-400">
          Have questions about our anonymous messaging service? We're here to help.
        </p>
        
        {#if sent}
          <div class="bg-green-50 dark:bg-green-900/30 border border-green-200 dark:border-green-700 rounded-md p-4 mb-6 text-green-800 dark:text-green-200">
            <div class="flex">
              <svg class="h-5 w-5 text-green-500" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
              <p class="ml-3">Thank you! Your message has been sent successfully.</p>
            </div>
          </div>
        {/if}
        
        {#if error}
          <div class="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded-md p-4 mb-6 text-red-800 dark:text-red-200">
            <div class="flex items-center">
              <svg class="h-5 w-5 text-red-500" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
              <p class="ml-3">{error}</p>
            </div>
          </div>
        {/if}
        
        <form onsubmit={handleSubmit} class="space-y-6">
          <div>
            <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Name</label>
            <div class="mt-1">
              <input
                id="name"
                type="text"
                bind:value={name}
                class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                placeholder="Your name"
              />
            </div>
          </div>
          
          <div>
            <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Email</label>
            <div class="mt-1">
              <input
                id="email"
                type="email"
                bind:value={email}
                class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                placeholder="your.email@example.com"
              />
            </div>
          </div>
          
          <div>
            <label for="message" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Message</label>
            <div class="mt-1">
              <textarea
                id="message"
                rows="5"
                bind:value={message}
                class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                placeholder="Your message here..."
              ></textarea>
            </div>
          </div>
          
          <div class="flex justify-end">
            <button
              type="submit"
              disabled={sending}
              class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 dark:focus:ring-offset-gray-900 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 cursor-pointer"
            >
              {#if sending}
                <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                Sending...
              {:else}
                Send Message
              {/if}
            </button>
          </div>
        </form>
      </div>
      
    </div>
    
  </div>
</div>