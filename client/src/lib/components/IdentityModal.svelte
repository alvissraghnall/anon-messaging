<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { enhance } from '$app/forms';

	let {
		show,
		isLoading,
		onConfirm,
		onCancel,
		username = $bindable(),
		password = $bindable(),
		handleSubmit,
		form
	} = $props();

	function stopPropagation<T extends Event>(fn: ((this: HTMLElement, event: T) => void) | null) {
		return function (this: HTMLElement, event: T) {
			event.stopPropagation();
			fn && fn.call(this, event);
		};
	}

	//	$inspect(username, password);
	//	console.log(form);

	function closeModalHandler(this: HTMLElement, event: Event) {
		onCancel?.();
	}

	/*
  export const handleEnhancedSubmit: SubmitFunction = async ({ formElement, formData, action, cancel, submitter }) => {
    
    const kp = await generateRSAKeyPair();
    const priv = forge.pki.privateKeyToPem(kp.privateKey);
    const pub = forge.pki.publicKeyToPem(kp.publicKey);
       
    const newFormData = new FormData();
    newFormData.set('username', username);
    newFormData.set('password', password);
    
    return async ({ result }) => {
      
    };
  }*/
</script>

{#if show}
	<div
		tabindex="0"
		role="dialog"
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		transition:fade
		onkeydown={(e) => (e.key === 'Enter' || e.key === 'Escape' || e.key === ' ') && onCancel()}
		onclick={stopPropagation(closeModalHandler)}
	>
		<div
			class="mx-4 w-full max-w-md rounded-lg bg-white p-6 shadow-lg dark:bg-gray-900"
			tabindex="-2"
			role="button"
			transition:scale={{ duration: 250 }}
			onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && void 0}
			onclick={stopPropagation(null)}
		>
			<h2 class="mb-4 text-xl font-semibold text-gray-900 dark:text-white">Create New Identity</h2>
			<p class="mb-4 text-sm text-gray-700 dark:text-gray-300">
				A secure key pair will be generated for you. You may enter an optional username but a secure
				password.
			</p>

			{#if form?.error}
				<ul class="my-4 rounded border border-red-400 bg-red-100 px-4 py-3 text-red-700">
					{#each form.errors as error}
						<li class="list-inside list-disc">{error.message}</li>
					{/each}
				</ul>
			{/if}

			<form id="create-identity" method="POST" use:enhance={handleSubmit}>
				<input
					type="text"
					placeholder="Optional Username"
					class="mb-4 w-full rounded-md border border-gray-300 bg-white px-4 py-2 text-gray-900 focus:ring-2 focus:ring-indigo-500 focus:outline-none dark:border-gray-700 dark:bg-gray-800 dark:text-white"
					bind:value={username}
				/>
				<input
					type="text"
					placeholder="Password"
					class="mb-4 w-full rounded-md border border-gray-300 bg-white px-4 py-2 text-gray-900 focus:ring-2 focus:ring-indigo-500 focus:outline-none dark:border-gray-700 dark:bg-gray-800 dark:text-white"
					bind:value={password}
					required
					aria-required="true"
				/>
				<div class="flex justify-end space-x-2">
					<button
						class="rounded bg-gray-200 px-4 py-2 text-gray-800 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
						onclick={() => onCancel?.()}
						type="button"
						disabled={isLoading}
					>
						Cancel
					</button>
					<button
						class="rounded bg-indigo-600 px-4 py-2 text-white hover:bg-indigo-700 disabled:opacity-50"
						disabled={isLoading}
						type="submit"
					>
						{#if isLoading}
							<span class="inline-flex items-center">
								<span
									data-testid='spinner'
									class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"
								></span>
								<span>Creating...</span>
							</span>
						{:else}
							<span>Confirm</span>
						{/if}
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}
