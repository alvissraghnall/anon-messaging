<script lang="ts">
  import { fade, scale } from "svelte/transition";
  let { show, isLoading, onConfirm, onCancel } = $props();
  let username = $state('');
  let password = $state('');

  function preventDefaultAndStopPropagation<T extends Event>(fn: ((this: HTMLElement, event: T) => void) | null) {
		return function (this: HTMLElement, event: T) {
			event.preventDefault();
			event.stopPropagation();
			fn && fn.call(this, event);
		};
	}

	function closeModalHandler(this: HTMLElement, event: Event) {
  	onCancel?.();
  }
</script>

{#if show}
  <div 
    tabindex="0"
  	role="button" 
  	class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" 
	  transition:fade
	  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && onCancel()}
	  onclick={preventDefaultAndStopPropagation(closeModalHandler)}
  >
    <div 
        class="bg-white dark:bg-gray-900 p-6 rounded-lg shadow-lg max-w-md w-full mx-4" 
        tabindex="-2"
        role="button"
        transition:scale={{ duration: 250 }}
    	  onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && void(0)}
        onclick={preventDefaultAndStopPropagation(null)}
    >
      <h2 class="text-xl font-semibold mb-4 text-gray-900 dark:text-white">Create New Identity</h2>
      <p class="text-sm text-gray-700 dark:text-gray-300 mb-4">
        A secure key pair will be generated for you. You may enter an optional username but a secure password.
      </p>
      <input
        type="text"
        placeholder="Optional Username"
        class="w-full px-4 py-2 mb-4 border border-gray-300 dark:border-gray-700 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
        bind:value={username}
      />
      <input
        type="text"
        placeholder="Password"
        class="w-full px-4 py-2 mb-4 border border-gray-300 dark:border-gray-700 rounded-md focus:outline-none focus:ring-2 focus:ring-indigo-500 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
        bind:value={password}
      />
      <div class="flex justify-end space-x-2">
        <button
          class="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
          onclick={() => onCancel?.()}
          disabled={isLoading}
        >
          Cancel
        </button>
        <button
          class="px-4 py-2 bg-indigo-600 text-white rounded hover:bg-indigo-700 disabled:opacity-50"
          onclick={() => onConfirm?.(username, password)}
          disabled={isLoading}
        >
          {#if isLoading}
            <span class="inline-flex items-center">
              <span class="animate-spin mr-2 border-2 border-white border-t-transparent rounded-full w-4 h-4"></span>
              <span>Creating...</span>
            </span>
          {:else}
            <span>Confirm</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
