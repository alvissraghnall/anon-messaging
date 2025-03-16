<script lang="ts">
	import { onMount } from 'svelte';
  import { browser } from '$app/environment';
  import { Icon, IconName } from '$lib';}
  import '../app.css';
	let { children } = $props();
	
	let darkMode = $state(false);
	function toggleDarkMode() {
    darkMode = !darkMode;
    localStorage.setItem('darkMode', darkMode.toString());
    updateTheme();
  }

  const loadSprite = () => {
    if (browser) {
      const spriteLink = document.createElement('link');
      spriteLink.rel = 'preload';
      spriteLink.href = '/sprite.svg';
      spriteLink.as = 'image';
      spriteLink.type = 'image/svg+xml';
      document.head.appendChild(spriteLink);
    }
  }

  onMount(() => {
    loadSprite();
    const storedPreference = localStorage.getItem('darkMode');
    if (storedPreference) {
      darkMode = storedPreference === 'true';
    } else {
      darkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    updateTheme();
  });

  function updateTheme() {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  }
</script>

<!-- Header -->
<header
	class="sticky top-0 z-10 bg-white shadow-md transition-colors duration-300 dark:bg-gray-800"
>
	<div class="container mx-auto flex items-center justify-between px-4 py-4">
		<div class="flex items-center">
			<div class="mr-2 text-2xl text-indigo-600 dark:text-indigo-400">🔐</div>
			<a
				href="/"
				class="text-xl font-bold text-gray-900 transition-colors duration-300 md:text-2xl dark:text-white"
				>Piree</a
			>
		</div>

		<div class="flex items-center space-x-4">
			<nav class="hidden space-x-6 font-semibold md:flex">
				<a
					href="/about"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
					>About</a
				>
				<a
					href="/features"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
					>Features</a
				>
				<a
					href="/security"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
					>Security</a
				>
				<a
					href="/contact"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
					>Contact</a
				>
			</nav>

			<button
				onclick={toggleDarkMode}
				class="rounded-lg bg-gray-200 p-2 text-gray-800 transition-colors duration-200 hover:cursor-pointer hover:bg-gray-300 focus:ring-2 focus:ring-gray-400 focus:outline-none dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
				aria-label="Toggle dark mode"
			>
				{#if darkMode}
					<!-- <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
            </svg> -->
					<Icon name={IconName.SUNLIGHT} size={24} />
				{:else}
					<!-- <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
            </svg> -->
					<Icon name={IconName.MOON} size={24} />
				{/if}
			</button>

			<a
				href="/login"
				class="hidden rounded-lg border border-indigo-600 px-4 py-2 text-indigo-600 transition-colors duration-200 hover:bg-indigo-50 md:inline-block dark:border-indigo-400 dark:text-indigo-400 dark:hover:bg-gray-800"
			>
				Sign In
			</a>
		</div>
	</div>
</header>

{@render children()}

<!-- Footer -->
<footer
	class="border-t border-gray-200 bg-white py-8 font-semibold transition-colors duration-300 dark:border-gray-700 dark:bg-gray-800"
>
	<div class="container mx-auto px-4">
		<div class="flex flex-col items-center justify-between space-y-4 md:flex-row md:space-y-0">
			<div class="text-gray-600 dark:text-gray-300">
				&copy; {new Date().getFullYear()} Piree. All rights reserved.
			</div>
			<div class="flex space-x-6">
				<a
					href="/privacy-policy"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
				>
					Privacy Policy
				</a>
				<a
					href="/terms"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
				>
					Terms of Service
				</a>
				<a
					href="/contact"
					class="text-gray-600 transition-colors duration-200 hover:text-indigo-600 dark:text-gray-300 dark:hover:text-indigo-400"
				>
					Contact Us
				</a>
			</div>
		</div>
	</div>
</footer>
