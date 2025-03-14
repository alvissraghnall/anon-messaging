<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { fly, fade } from 'svelte/transition';
  import { spring } from 'svelte/motion';
  
  let isLoading = false;
  let error = '';
  let darkMode = false;
  let mounted = false;

  // Animation helpers
  const staggerDelay = 150;
  
  onMount(() => {
    // Check for dark mode preference in localStorage
    if (localStorage.getItem('darkMode') === 'true') {
      darkMode = true;
      document.documentElement.classList.add('dark');
    }
    mounted = true;
  });
  
  function toggleDarkMode() {
    darkMode = !darkMode;
    localStorage.setItem('darkMode', darkMode.toString());
    
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
  
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

<div class="min-h-screen bg-gray-100 dark:bg-gray-900 transition-colors duration-300">
  <!-- Header -->
  <header class="bg-white dark:bg-gray-800 shadow-md sticky top-0 z-10 transition-colors duration-300">
    <div class="container mx-auto px-4 py-4 flex items-center justify-between">
      <div class="flex items-center">
        <div class="text-indigo-600 dark:text-indigo-400 text-2xl mr-2">🔐</div>
        <a href="/" class="text-xl md:text-2xl font-bold text-gray-900 dark:text-white transition-colors duration-300">Piree</a>
      </div>
      
      <div class="flex items-center space-x-4">
        <nav class="hidden md:flex space-x-6">
          <a href="/about" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">About</a>
          <a href="/features" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">Features</a>
          <a href="/security" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">Security</a>
          <a href="/contact" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">Contact</a>
        </nav>
        
        <button 
          onclick={toggleDarkMode}
          class="p-2 rounded-lg bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-gray-400 transition-colors duration-200 hover:cursor-pointer"
          aria-label="Toggle dark mode"
        >
          {#if darkMode}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
            </svg>
          {/if}
        </button>
        
        <a href="/login" class="hidden md:inline-block px-4 py-2 border border-indigo-600 text-indigo-600 dark:text-indigo-400 dark:border-indigo-400 rounded-lg hover:bg-indigo-50 dark:hover:bg-gray-800 transition-colors duration-200">
          Sign In
        </a>
      </div>
    </div>
  </header>

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
              <span class="inline-block animate-spin mr-2">⟳</span>
              Creating Identity...
            {:else}
              Create New Identity
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
    
    <!-- Chat Illustration -->
    {#if mounted}
      <div class="max-w-4xl mx-auto mb-24" in:fade={{ duration: 800, delay: 450 }}>
        <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-lg transition-colors duration-300">
          <div class="flex flex-col space-y-4">
            <!-- Chat header -->
            <div class="flex items-center pb-3 border-b border-gray-200 dark:border-gray-700">
              <div class="w-10 h-10 rounded-full bg-indigo-100 dark:bg-indigo-900 flex items-center justify-center text-indigo-600 dark:text-indigo-400">A</div>
              <div class="ml-3">
                <p class="text-gray-900 dark:text-white font-medium">Anonymous Recipient</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">Online • End-to-end encrypted</p>
              </div>
            </div>
            
            <!-- Chat messages -->
            <div class="space-y-3">
              <div class="flex items-end">
                <div class="bg-gray-100 dark:bg-gray-700 p-3 rounded-lg rounded-bl-none max-w-xs transition-colors duration-300">
                  <p class="text-gray-800 dark:text-gray-200">Hi there! I need to share some sensitive information with you.</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">10:24 AM</p>
                </div>
              </div>
              
              <div class="flex items-end justify-end">
                <div class="bg-indigo-600 p-3 rounded-lg rounded-br-none max-w-xs transition-colors duration-300">
                  <p class="text-white">No problem. Our messages are fully encrypted, so only we can read them.</p>
                  <p class="text-xs text-indigo-200 mt-1">10:26 AM</p>
                </div>
              </div>
              
              <div class="flex items-end">
                <div class="bg-gray-100 dark:bg-gray-700 p-3 rounded-lg rounded-bl-none max-w-xs transition-colors duration-300">
                  <p class="text-gray-800 dark:text-gray-200">Great. I'll set this message to self-destruct after you read it.</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">10:27 AM</p>
                </div>
              </div>
              
              <div class="flex items-end justify-end">
                <div class="bg-indigo-600 p-3 rounded-lg rounded-br-none max-w-xs transition-colors duration-300">
                  <p class="text-white">Perfect. I'll check it as soon as I can. No one else will be able to see it.</p>
                  <p class="text-xs text-indigo-200 mt-1">10:28 AM</p>
                </div>
              </div>
            </div>
            
            <!-- Chat input -->
            <div class="flex items-center mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
              <input
                type="text"
                placeholder="Type your message..."
                class="flex-grow bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-white p-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 transition-colors duration-300"
              />
              <button class="ml-2 p-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors duration-300 hover:cursor-pointer">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    {/if}
    
    <!-- Features Section -->
    {#if mounted}
      <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto mb-24">
        {#each [
          { 
            icon: "🔒", 
            title: "End-to-End Encryption", 
            description: "All messages are encrypted with state-of-the-art asymmetric cryptography, ensuring only the intended recipient can read them." 
          },
          { 
            icon: "👤", 
            title: "Total Anonymity", 
            description: "No personally identifiable information is ever collected. Your identity is protected by cryptographic keypairs." 
          },
          { 
            icon: "🔥", 
            title: "Self-Destructing Messages", 
            description: "Set your messages to automatically delete after being read or after a specific time period." 
          }
        ] as feature, i}
          <div 
            class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-lg hover:shadow-xl transform transition-all duration-300 hover:-translate-y-1 cursor-pointer"
            in:fly={{ y: 20, duration: 800, delay: 600 + (i * staggerDelay) }}
          >
            <div class="text-indigo-600 dark:text-indigo-400 text-2xl mb-4">{feature.icon}</div>
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
          class="px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 transition-all duration-300 transform hover:scale-105 hover:cursor-pointer"
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

  <!-- Footer -->
  <footer class="bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 py-8 transition-colors duration-300 font-semibold">
    <div class="container mx-auto px-4">
      <div class="flex flex-col md:flex-row justify-between items-center space-y-4 md:space-y-0">
        <div class="text-gray-600 dark:text-gray-300">
          &copy; {new Date().getFullYear()} Piree. All rights reserved.
        </div>
        <div class="flex space-x-6">
          <a href="/privacy" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">
            Privacy Policy
          </a>
          <a href="/terms" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">
            Terms of Service
          </a>
          <a href="/contact" class="text-gray-600 dark:text-gray-300 hover:text-indigo-600 dark:hover:text-indigo-400 transition-colors duration-200">
            Contact Us
          </a>
        </div>
      </div>
    </div>
  </footer>
</div>