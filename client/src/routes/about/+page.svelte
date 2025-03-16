<script lang="ts">
  import { onMount } from 'svelte';

  // Dark mode state
  let darkMode: boolean;

  // Initialize dark mode based on system preference or stored preference
  onMount(() => {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      darkMode = savedTheme === 'dark';
    } else {
      darkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    applyTheme();
  });

  // Toggle dark mode
  function toggleDarkMode() {
    darkMode = !darkMode;
    localStorage.setItem('theme', darkMode ? 'dark' : 'light');
    applyTheme();
  }

  // Apply theme to document
  function applyTheme() {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
</script>

<svelte:head>
  <title>About - Anonymous Messaging Service</title>
  <meta name="description" content="Secure, anonymous messaging with end-to-end encryption and zero tracking." />
</svelte:head>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-200">
  <div class="fixed top-4 right-4 z-10">
    <button 
      on:click={toggleDarkMode}
      class="p-2 rounded-full bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 transition-colors duration-200"
      aria-label={darkMode ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {#if darkMode}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
        </svg>
      {/if}
    </button>
  </div>

  <header class="pt-16 pb-8 px-4 md:px-8 text-center">
    <h1 class="text-4xl md:text-5xl font-bold text-gray-900 dark:text-white mb-2">Anonymous Messaging</h1>
    <p class="text-xl text-indigo-600 dark:text-indigo-400">Truly private communication for everyone</p>
  </header>

  <main class="max-w-4xl mx-auto px-4 md:px-8 pb-16">
    <section class="mb-16">
      <h2 class="text-2xl md:text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">What We're About</h2>
      <div class="prose dark:prose-invert prose-lg max-w-none">
        <p>
          We believe privacy is a fundamental human right. Our platform enables truly anonymous communication 
          using asymmetric cryptography, allowing you to send and receive messages without revealing your identity.
        </p>
        <p>
          Unlike other messaging services that claim to be "private" but still collect metadata or require phone numbers,
          our service is built from the ground up with zero knowledge principles. We simply cannot track you,
          even if we wanted to.
        </p>
      </div>
    </section>

    <section class="mb-16">
      <h2 class="text-2xl md:text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">How It Works</h2>
      <div class="grid md:grid-cols-3 gap-8">
        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
          <div class="text-indigo-600 dark:text-indigo-400 mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">Asymmetric Keys</h3>
          <p class="text-gray-600 dark:text-gray-300">
            When you create an address, we generate a unique keypair. Your public key becomes your address, while your private key
            remains client-side to decrypt messages.
          </p>
        </div>

        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
          <div class="text-indigo-600 dark:text-indigo-400 mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">End-to-End Encryption</h3>
          <p class="text-gray-600 dark:text-gray-300">
            Messages are encrypted on your device before transmission. Only the recipient with the correct private key can decrypt and read them.
          </p>
        </div>

        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-md">
          <div class="text-indigo-600 dark:text-indigo-400 mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">Zero Knowledge</h3>
          <p class="text-gray-600 dark:text-gray-300">
            We never store IP addresses, don't require emails or phone numbers, and can't read your messages. Total anonymity by design.
          </p>
        </div>
      </div>
    </section>

    <section class="mb-16">
      <h2 class="text-2xl md:text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">Technical Details</h2>
      <div class="prose dark:prose-invert prose-lg max-w-none">
        <p>
          Our service employs battle-tested cryptographic principles:
        </p>
        <ul>
          <li>
            <strong>Elliptic Curve Cryptography</strong> for generating keypairs that are both secure and efficient
          </li>
          <li>
            <strong>AES-256</strong> for symmetric message encryption
          </li>
          <li>
            <strong>HMAC signatures</strong> to verify message integrity
          </li>
          <li>
            <strong>Secure client-side operations</strong> - all encryption/decryption happens in your browser
          </li>
        </ul>
        <p>
          Our backend microservice is purpose-built using Rust for maximum security and performance, while our frontend uses Svelte 5 
          with TypeScript for a responsive, type-safe user experience.
        </p>
      </div>
    </section>

    <section>
      <h2 class="text-2xl md:text-3xl font-bold text-gray-800 dark:text-gray-100 mb-6">Why Use Our Service?</h2>
      <div class="bg-indigo-100 dark:bg-indigo-900/30 p-6 rounded-lg">
        <div class="grid md:grid-cols-2 gap-6">
          <div>
            <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">For Individuals</h3>
            <ul class="space-y-2 text-gray-700 dark:text-gray-300">
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Communicate without fear of surveillance</span>
              </li>
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Share sensitive information securely</span>
              </li>
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Protect your digital identity</span>
              </li>
            </ul>
          </div>
          <div>
            <h3 class="text-xl font-semibold text-gray-800 dark:text-white mb-2">For Organizations</h3>
            <ul class="space-y-2 text-gray-700 dark:text-gray-300">
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Create secure feedback channels</span>
              </li>
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Enable whistleblower protection</span>
              </li>
              <li class="flex items-start">
                <svg class="h-6 w-6 text-green-500 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                <span>Communicate with sources confidentially</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </section>
  </main>

</div>