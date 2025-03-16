<script lang="ts">
	import '../app.css';
	let { children } = $props();
	
	let darkMode = false;
	function toggleDarkMode() {
		darkMode = !darkMode;
		localStorage.setItem('darkMode', darkMode.toString());
		
		if (darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	}
</script>

<!-- Header -->
<header class="bg-white dark:bg-gray-800 shadow-md sticky top-0 z-10 transition-colors duration-300">
    <div class="container mx-auto px-4 py-4 flex items-center justify-between">
      <div class="flex items-center">
        <div class="text-indigo-600 dark:text-indigo-400 text-2xl mr-2">🔐</div>
        <a href="/" class="text-xl md:text-2xl font-bold text-gray-900 dark:text-white transition-colors duration-300">Piree</a>
      </div>
      
      <div class="flex items-center space-x-4">
        <nav class="hidden md:flex space-x-6 font-semibold">
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

{@render children()}

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