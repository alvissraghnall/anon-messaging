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

			await new Promise((resolve) => setTimeout(resolve, 10000));

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
	<meta name="description" content="Contact the team behind Piree anonymous messaging service" />
</svelte:head>

<div
	class="min-h-screen bg-gray-50 text-gray-900 transition-colors duration-200 dark:bg-gray-900 dark:text-gray-100"
>
	<div class="container mx-auto max-w-4xl px-4 py-16">
		<div class="overflow-hidden rounded-lg bg-white shadow-lg dark:bg-gray-800">
			<div class="px-6 py-8 md:p-10">
				<h1 class="mb-2 text-center text-3xl font-bold text-gray-900 dark:text-gray-100">
					Contact Us
				</h1>
				<p class="mb-8 text-center text-gray-600 dark:text-gray-400">
					Have questions about our anonymous messaging service? We're here to help.
				</p>

				{#if sent}
					<div
						class="mb-6 rounded-md border border-green-200 bg-green-50 p-4 text-green-800 dark:border-green-700 dark:bg-green-900/30 dark:text-green-200"
					>
						<div class="flex">
							<svg class="h-5 w-5 text-green-500" viewBox="0 0 20 20" fill="currentColor">
								<path
									fill-rule="evenodd"
									d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
									clip-rule="evenodd"
								/>
							</svg>
							<p class="ml-3">Thank you! Your message has been sent successfully.</p>
						</div>
					</div>
				{/if}

				{#if error}
					<div
						class="mb-6 rounded-md border border-red-200 bg-red-50 p-4 text-red-800 dark:border-red-700 dark:bg-red-900/30 dark:text-red-200"
					>
						<div class="flex items-center">
							<svg class="h-5 w-5 text-red-500" viewBox="0 0 20 20" fill="currentColor">
								<path
									fill-rule="evenodd"
									d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
									clip-rule="evenodd"
								/>
							</svg>
							<p class="ml-3">{error}</p>
						</div>
					</div>
				{/if}

				<form onsubmit={handleSubmit} class="space-y-6">
					<div>
						<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300"
							>Name</label
						>
						<div class="mt-1">
							<input
								id="name"
								type="text"
								bind:value={name}
								class="block w-full rounded-md border-gray-300 bg-white px-3 py-2 text-gray-900 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
								placeholder="Your name"
							/>
						</div>
					</div>

					<div>
						<label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300"
							>Email</label
						>
						<div class="mt-1">
							<input
								id="email"
								type="email"
								bind:value={email}
								class="block w-full rounded-md border-gray-300 bg-white px-3 py-2 text-gray-900 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
								placeholder="your.email@example.com"
							/>
						</div>
					</div>

					<div>
						<label for="message" class="block text-sm font-medium text-gray-700 dark:text-gray-300"
							>Message</label
						>
						<div class="mt-1">
							<textarea
								id="message"
								rows="5"
								bind:value={message}
								class="block w-full rounded-md border-gray-300 bg-white px-3 py-2 text-gray-900 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100"
								placeholder="Your message here..."
							></textarea>
						</div>
					</div>

					<div class="flex justify-end">
						<button
							type="submit"
							disabled={sending}
							class="inline-flex cursor-pointer justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors duration-200 hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 dark:focus:ring-offset-gray-900"
						>
							{#if sending}
								<svg
									class="mr-3 -ml-1 h-5 w-5 animate-spin text-white"
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
								>
									<circle
										class="opacity-25"
										cx="12"
										cy="12"
										r="10"
										stroke="currentColor"
										stroke-width="4"
									></circle>
									<path
										class="opacity-75"
										fill="currentColor"
										d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
									></path>
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
