<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  let isLoading = false;
  let error = '';
  
  async function createNewIdentity() {
    isLoading = true;
    error = '';
    
    try {
      // Call to your Rust microservice to generate keypair
      const response = await fetch('/api/identity/create', {
        method: 'POST',
      });
      
      if (!response.ok) {
        throw new Error('Failed to create identity');
      }
      
      const data = await response.json();
      
      localStorage.setItem('privateKey', data.privateKey);
      localStorage.setItem('publicKey', data.publicKey);
      
      // Navigate to the dashboard
      goto('/dashboard');
    } catch (err) {
      error = err instanceof Error ? err.message : 'Something went wrong';
    } finally {
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen bg-gray-100 dark:bg-gray-900">
  <main class="container mx-auto px-4 py-16">
    <!-- Hero Section -->
    <div class="max-w-3xl mx-auto text-center mb-16">
      <h1 class="text-4xl md:text-6xl font-bold mb-6 text-gray-900 dark:text-white">
        Send Messages. Stay Anonymous.
      </h1>
      <p class="text-xl text-gray-600 dark:text-gray-300 mb-10">
        A secure platform for anonymous communication, powered by asymmetric cryptography.
      </p>
      
      <div class="flex flex-col sm:flex-row gap-4 justify-center">
        <button 
          on:click={createNewIdentity}
          class="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 transition-colors"
          disabled={isLoading}
        >
          {#if isLoading}
            <span class="inline-block animate-spin mr-2">⟳</span>
            Creating Identity...
          {:else}
            Create New Identity
          {/if}
        </button>
        
        <a href="/login" class="px-6 py-3 border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 transition-colors">
          Access Existing Identity
        </a>
      </div>
      
      {#if error}
        <p class="mt-4 text-red-600 dark:text-red-400">{error}</p>
      {/if}
    </div>
    
    <!-- Features Section -->
    <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <div class="text-indigo-600 dark:text-indigo-400 text-2xl mb-4">🔒</div>
        <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">End-to-End Encryption</h3>
        <p class="text-gray-600 dark:text-gray-300">
          All messages are encrypted with state-of-the-art asymmetric cryptography, ensuring only the intended recipient can read them.
        </p>
      </div>
      
      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <div class="text-indigo-600 dark:text-indigo-400 text-2xl mb-4">👤</div>
        <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Total Anonymity</h3>
        <p class="text-gray-600 dark:text-gray-300">
          No personally identifiable information is ever collected. Your identity is protected by cryptographic keypairs.
        </p>
      </div>
      
      <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
        <div class="text-indigo-600 dark:text-indigo-400 text-2xl mb-4">🔥</div>
        <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Self-Destructing Messages</h3>
        <p class="text-gray-600 dark:text-gray-300">
          Set your messages to automatically delete after being read or after a specific time period.
        </p>
      </div>
    </div>
    
    <!-- How It Works Section -->
    <div class="max-w-4xl mx-auto mt-20">
      <h2 class="text-3xl font-bold text-center mb-10 text-gray-900 dark:text-white">How It Works</h2>
      
      <div class="space-y-12">
        <div class="flex flex-col md:flex-row items-center gap-6">
          <div class="md:w-1/3 flex justify-center">
            <div class="w-16 h-16 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-2xl text-indigo-600 dark:text-indigo-400">1</div>
          </div>
          <div class="md:w-2/3">
            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Create Your Identity</h3>
            <p class="text-gray-600 dark:text-gray-300">
              Generate a unique cryptographic keypair that serves as your anonymous identity on the platform.
            </p>
          </div>
        </div>
        
        <div class="flex flex-col md:flex-row items-center gap-6">
          <div class="md:w-1/3 flex justify-center">
            <div class="w-16 h-16 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-2xl text-indigo-600 dark:text-indigo-400">2</div>
          </div>
          <div class="md:w-2/3">
            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Share Your Public Address</h3>
            <p class="text-gray-600 dark:text-gray-300">
              Share your public key with others so they can send you encrypted messages that only you can decrypt.
            </p>
          </div>
        </div>
        
        <div class="flex flex-col md:flex-row items-center gap-6">
          <div class="md:w-1/3 flex justify-center">
            <div class="w-16 h-16 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-2xl text-indigo-600 dark:text-indigo-400">3</div>
          </div>
          <div class="md:w-2/3">
            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">Communicate Securely</h3>
            <p class="text-gray-600 dark:text-gray-300">
              Send and receive encrypted messages without revealing your identity. Only the recipient with the correct private key can decrypt and read your messages.
            </p>
          </div>
        </div>
      </div>
    </div>
    
    <!-- FAQ Section -->
    <div class="max-w-3xl mx-auto mt-20">
      <h2 class="text-3xl font-bold text-center mb-10 text-gray-900 dark:text-white">Frequently Asked Questions</h2>
      
      <div class="space-y-6">
        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <h3 class="text-lg font-semibold mb-2 text-gray-900 dark:text-white">How secure is this service?</h3>
          <p class="text-gray-600 dark:text-gray-300">
            We use industry-standard asymmetric encryption implemented in Rust. Your private keys never leave your device, ensuring maximum security.
          </p>
        </div>
        
        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <h3 class="text-lg font-semibold mb-2 text-gray-900 dark:text-white">What if I lose my private key?</h3>
          <p class="text-gray-600 dark:text-gray-300">
            Unfortunately, if you lose your private key, you'll lose access to your messages and identity. We recommend securely backing up your keys.
          </p>
        </div>
        
        <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow">
          <h3 class="text-lg font-semibold mb-2 text-gray-900 dark:text-white">Are messages stored on your servers?</h3>
          <p class="text-gray-600 dark:text-gray-300">
            Encrypted messages are temporarily stored until they are delivered to the recipient or until they self-destruct. We cannot read the content of these messages.
          </p>
        </div>
      </div>
    </div>
  </main>
  
  <!-- Footer -->
  <footer class="bg-white dark:bg-gray-800 py-6 mt-16">
    <div class="container mx-auto px-4 text-center text-gray-600 dark:text-gray-400">
      <p>© {new Date().getFullYear()} Anonymous Messaging Service</p>
    </div>
  </footer>
</div>