<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { flip } from "svelte/animate";
  import { fly, fade, slide,  } from 'svelte/transition';
  import { spring, } from 'svelte/motion';
  import { Icon, IconName, IdentityModal, darkMode } from '$lib';
//  import { darkMode } from '$lib/stores/theme';

  let isLoading = false;
  let error = '';
  let mounted = $state(false);

  let showModal = $state(false);
  let modalLoading = $state(false);
  let modalError = $state('');

  const staggerDelay = 150;

  onMount(() => {
    mounted = true;
  });

  function createNewIdentity() {
    showModal = true;
  }

  function handleConfirm(username: string) {
    modalLoading = true;
    console.log("Creating identity for:", username || "Anonymous");

    setTimeout(() => {
      modalLoading = false;
      showModal = false;
    }, 2000);
  }

  function handleCancel() {
    showModal = false;
  }
</script>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 transition-colors duration-300">

  <main class="container mx-auto px-4 py-16">
    <!-- Hero Section -->
    {#if mounted}
      <div class="max-w-3xl mx-auto text-center mb-20" in:fade={{ duration: 800, delay: 300 }}>
        <h1 class="text-4xl md:text-6xl font-bold mb-6 text-gray-900 dark:text-white transition-colors duration-300">
          Send Messages. Stay Anonymous.
        </h1>
        <p class="text-xl text-gray-600 dark:text-gray-300 mb-10 transition-colors duration-300">
          A secure platform for anonymous communication, powered by asymmetric cryptography.
        </p>

        <div class="flex flex-col sm:flex-row gap-4 justify-center">
          <button
            onclick={createNewIdentity}
            class="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 transition-all duration-300 transform hover:scale-105 hover:cursor-pointer"
            disabled={isLoading}
          >
            {#if isLoading}
		      <span class="inline-flex items-center">
		        <span class="animate-spin mr-2 border-2 border-white border-t-transparent rounded-full w-4 h-4"></span>
		        <span>Creating Identity...</span>
		      </span>
		    {:else}
		      <span>Create New Identity</span>
		    {/if}
          </button>

          <a href="/login" class="px-6 py-3 border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-all duration-300 transform hover:scale-105">
            Access Existing Identity
          </a>
        </div>

        {#if error}
          <p class="mt-4 text-red-600 dark:text-red-400 transition-colors duration-300">{error}</p>
        {/if}
      </div>
    {/if}

    <IdentityModal
      show={showModal}
      isLoading={modalLoading}
      onConfirm={handleConfirm}
      onCancel={handleCancel}
    />

    {#if mounted}
  <div class="max-w-4xl mx-auto mb-24"
       transition:fly={{ y: 20, duration: 600, }}>
    <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-lg transition-all duration-300 hover:shadow-xl">
      <div class="flex flex-col space-y-4">
        <!-- Chat header with animation -->
        <div class="flex items-center pb-3 border-b border-gray-200 dark:border-gray-700 transition-all duration-300"
             in:slide={{ duration: 400, delay: 500 }}>
          <div class="w-10 h-10 rounded-full bg-emerald-100 dark:bg-emerald-900 flex items-center justify-center text-emerald-600 dark:text-emerald-400 transition-colors duration-300 transform hover:scale-105">
            <span class="text-lg font-medium">A</span>
          </div>
          <div class="ml-3">
            <p class="text-gray-900 dark:text-white font-medium">Anonymous Recipient</p>
            <p class="text-xs text-gray-500 dark:text-gray-400 flex items-center">
              <span class="inline-block w-2 h-2 bg-emerald-500 rounded-full mr-1 animate-pulse"></span>
              Online • End-to-end encrypted
            </p>
          </div>

          <!-- Settings button -->
          <button class="ml-auto p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors duration-200 rounded-full hover:bg-gray-100 dark:hover:bg-gray-700">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
            </svg>
          </button>
        </div>

        <!-- Chat messages with staggered animations -->
        <div class="space-y-3 overflow-y-auto max-h-96 py-2 scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600 scrollbar-track-transparent">
          {#each [
            { sender: 'them', text: "Hi there! I need to share some sensitive information with you.", time: "10:24 AM", delay: 600 },
            { sender: 'me', text: "No problem. Our messages are fully encrypted, so only we can read them.", time: "10:26 AM", delay: 750 },
            { sender: 'them', text: "Great. I'll set this message to self-destruct after you read it.", time: "10:27 AM", delay: 900 },
            { sender: 'me', text: "Perfect. I'll check it as soon as I can. No one else will be able to see it.", time: "10:28 AM", delay: 1050 }
          ] as message, index (message.time)}
            <div class="flex items-end {message.sender === 'me' ? 'justify-end' : ''}"
              in:fade={{ duration: 400, delay: message.delay }}
            >
              <div class="{message.sender === 'me' ?
                         'bg-emerald-600 text-white' :
                         'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200'}
                         p-3 rounded-lg {message.sender === 'me' ? 'rounded-br-none' : 'rounded-bl-none'}
                         max-w-xs shadow-sm transition-all duration-300 hover:shadow-md transform hover:scale-[1.01]">
                <p>{message.text}</p>
                <p class="text-xs {message.sender === 'me' ? 'text-emerald-200' : 'text-gray-500 dark:text-gray-400'} mt-1">{message.time}</p>
              </div>
            </div>
          {/each}
        </div>

        <!-- New message indicator -->
        <div class="text-center text-xs text-gray-500 py-1 animate-pulse" in:fade={{ duration: 300, delay: 1200 }}>
          New messages
        </div>

        <!-- Chat input with animations -->
        <div class="flex items-center mt-3 pt-3 border-t border-gray-200 dark:border-gray-700 transition-all duration-300" in:slide={{ duration: 400, delay: 1100, axis: 'y' }}>
          <button class="p-2 text-gray-500 hover:text-emerald-600 dark:text-gray-400 dark:hover:text-emerald-400 transition-colors duration-200">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
            </svg>
          </button>

          <input
            type="text"
            placeholder="Type your message..."
            class="flex-grow mx-2 bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white p-3 rounded-lg focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all duration-300 placeholder-gray-500 dark:placeholder-gray-400"
          />

          <button class="p-3 bg-emerald-600 text-white rounded-lg hover:bg-emerald-700 transition-all duration-300 transform hover:scale-105 hover:rotate-1 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Connection status indicator -->
    <div class="mt-2 text-center text-xs text-gray-500 dark:text-gray-400 flex items-center justify-center space-x-1"
         in:fade={{ duration: 300, delay: 1300 }}>
      <span class="inline-block w-2 h-2 bg-emerald-500 rounded-full animate-pulse"></span>
      <span>Secure connection active</span>
    </div>
  </div>
{/if}

    <!-- Features Section -->
    {#if mounted}
      <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto mb-24">
        {#each [
          {
            icon: IconName.LOCK,
            title: "End-to-End Encryption",
            description: "All messages are encrypted with state-of-the-art asymmetric cryptography, ensuring only the intended recipient can read them."
          },
          {
            icon: IconName.USERSECRET,
            title: "Total Anonymity",
            description: "No personally identifiable information is ever collected. Your identity is protected by cryptographic keypairs."
          },
          {
            icon: IconName.CHATBUBBLELEFTRIGHT,
            title: "Reply Anonymous Messages",
            description: "Respond to messages with fun, anonymous replies and keep conversations flowing."
          }
        ] as feature, i}
          <div
            class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-lg hover:shadow-xl transform transition-all duration-300 hover:-translate-y-1 cursor-pointer"
            in:fly={{ y: 20, duration: 800, delay: 600 + (i * staggerDelay) }}
          >
            <div class="text-indigo-600 dark:text-indigo-400 text-2xl mb-4"><Icon name={feature.icon} size={24} color={feature.icon === IconName.USERSECRET && $darkMode ? "#7c86ff" : "#4f39f6" } /></div>
            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white transition-colors duration-300">{feature.title}</h3>
            <p class="text-gray-600 dark:text-gray-300 transition-colors duration-300">
              {feature.description}
            </p>
          </div>
        {/each}
      </div>
    {/if}

    <!-- How It Works Section -->
    {#if mounted}
      <div class="max-w-4xl mx-auto my-24" in:fade={{ duration: 800, delay: 900 }}>
        <h2 class="text-3xl font-bold text-center mb-10 text-gray-900 dark:text-white transition-colors duration-300">How It Works</h2>

        <div class="space-y-12">
          {#each [
            {
              title: "Create Your Identity",
              description: "Generate a unique cryptographic keypair that serves as your anonymous identity on the platform."
            },
            {
              title: "Share Your Public Address",
              description: "Share your public key with others so they can send you encrypted messages that only you can decrypt."
            },
            {
              title: "Communicate Securely",
              description: "Send and receive encrypted messages without revealing your identity. Only the recipient with the correct private key can decrypt and read your messages."
            }
          ] as step, i}
            <div class="flex flex-col md:flex-row items-center gap-6 transform transition-all duration-500 hover:scale-102" in:fly={{ x: i % 2 === 0 ? -20 : 20, duration: 800, delay: 1000 + (i * staggerDelay) }}>
              <div class="md:w-1/3 flex justify-center">
                <div class="w-16 h-16 rounded-full bg-indigo-600 dark:bg-indigo-700 flex items-center justify-center text-white text-2xl font-bold">
                  {i + 1}
                </div>
              </div>
              <div class="md:w-2/3">
                <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white transition-colors duration-300">
                  {step.title}
                </h3>
                <p class="text-gray-600 dark:text-gray-300 transition-colors duration-300">
                  {step.description}
                </p>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Call to Action Section -->
    {#if mounted}
      <div class="max-w-3xl mx-auto text-center my-24" in:fade={{ duration: 800, delay: 1200 }}>
        <h2 class="text-3xl font-bold mb-6 text-gray-900 dark:text-white transition-colors duration-300">
          Ready to Communicate Anonymously?
        </h2>
        <p class="text-xl text-gray-600 dark:text-gray-300 mb-10 transition-colors duration-300">
          Join thousands of users who trust Piree for secure, anonymous communication.
        </p>
        <button
          onclick={createNewIdentity}
          class="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-white hover:text-indigo-700 hover:outline-indigo-700 hover:outline-2 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 transition-all duration-300 transform hover:scale-105 hover:cursor-pointer"
          disabled={isLoading}
        >
          {#if isLoading}
            <span class="inline-block animate-spin mr-2">⟳</span>
            Creating Identity...
          {:else}
            Get Started Now
          {/if}
        </button>
      </div>
    {/if}
  </main>

</div>
