<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { flip } from 'svelte/animate';
	import { fly, fade, slide } from 'svelte/transition';
	import { spring } from 'svelte/motion';
	import { Icon, IconName, IdentityModal, darkMode } from '$lib';
	import { generateRSAKeyPair } from '$lib/utils/rsa-keygen';
	import * as forge from 'node-forge';
	import type { SubmitFunction, PageProps } from './$types';
	import { writable } from 'svelte/store';
	import { base64ToUrlSafe } from '$lib/utils/base64-to-url-safe';
	import { applyAction } from '$app/forms';

	let { form, data }: PageProps = $props();
	console.log(form);
	console.log(data);

	let isLoading = false;
	let error = '';
	let mounted = $state(false);

	let showModal = $state(false);
	let modalLoading = $state(false);
	let modalError = $state('');
	let username = $state('');
	let password = $state('');

	const staggerDelay = 150;

	onMount(() => {
		mounted = true;
	});

	$inspect(username, password);

	function publicKeyToUrlSafeBase64(publicKey: forge.pki.PublicKey) {
		const pem = forge.pki.publicKeyToPem(publicKey);

		// Strip PEM headers/footers and newlines	
		const base64 = pem
			.replace('-----BEGIN PUBLIC KEY-----', '')
			.replace('-----END PUBLIC KEY-----', '')
			.replace(/\r?\n|\r/g, '');

		// Convert to URL-safe Base64
		const urlSafeBase64 = base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');

		return urlSafeBase64;
	}

	function urlSafeBase64ToPublicKey(urlSafeBase64: string) {
		// Convert back to standard Base64
		let base64 = urlSafeBase64.replace(/-/g, '+').replace(/_/g, '/');

		// Add padding if needed
		const padding = base64.length % 4;
		if (padding > 0) {
			base64 += '='.repeat(4 - padding);
		}

		// Reconstruct PEM format
		const pem = `-----BEGIN PUBLIC KEY-----\n${base64.match(/.{1,64}/g)?.join('\n')}\n-----END PUBLIC KEY-----`;

		// Parse to forge public key object
		return forge.pki.publicKeyFromPem(pem);
	}

	function createNewIdentity() {
		showModal = true;
	}

	enum FormState {
		IDLE = 'idle',
		SUBMITTING = 'submitting',
		SUCCESS = 'success',
		ERROR = 'error',
		VALIDATION_ERROR = 'validation_error'
	}

	export const formStatus = writable<FormState>(FormState.IDLE);
	export const formErrors = writable({});
	export const generalError = writable('');

	export const handleEnhancedSubmit: SubmitFunction = async ({
		formElement,
		formData,
		action,
		cancel,
		submitter
	}) => {
		console.log('submitting');
		let newFormData: FormData;
		// Reset errors and set submitting state
		formStatus.set(FormState.SUBMITTING);
		formErrors.set({});
		generalError.set('');

		try {
			const kp = await generateRSAKeyPair();
			const priv = forge.pki.privateKeyToPem(kp.privateKey);
			const pubPem = forge.pki.publicKeyToPem(kp.publicKey);
			const pubB64 = publicKeyToUrlSafeBase64(kp.publicKey);

			newFormData = new FormData();
			console.log('username', username, password);
			newFormData.set('username', username);
			newFormData.set('password', password);
			newFormData.set('public_key', pubB64);

			// Store private key for later use

			formData.set('username', username);
			formData.set('password', password);
			formData.set('public_key', pubB64);
			// formData = newFormData;
		} catch (error) {
			formStatus.set(FormState.ERROR);
			generalError.set('Failed to generate security keys. Please try again.');
			cancel();
			return;
		}

		return async ({ result, formData, update }) => {
			update({ reset: false });
			// formData = newFormData;
			console.log('form', ...formData);
			console.log('result', result);

			if (result.type === 'success') {
				formStatus.set(FormState.SUCCESS);

				const { data }: any = result;

				if (data?.token) {
					localStorage.setItem('authToken', data.token);
				}

				if (data?.redirect) {
					await goto(data.redirect);
				} else {
					// await goto('/dashboard');
				}
			} else if (result.type === 'error') {
				formStatus.set(FormState.ERROR);
				generalError.set(result.error?.message || 'An error occurred during submission');
			} else if (result.type === 'failure') {
				formStatus.set(FormState.VALIDATION_ERROR);

				const { data } = result;
				if (data) {
					formErrors.set(data);
				}
			}
			applyAction(result);
		};
	};

	async function handleConfirm(username: string, password: string) {
		modalLoading = true;
		console.log('Creating identity for:', username || 'Anonymous');

		setTimeout(() => {
			modalLoading = false;
			showModal = false;
		}, 2000);
	}

	function handleCancel() {
		showModal = false;
	}
</script>

<div class="min-h-screen bg-gray-100 transition-colors duration-300 dark:bg-gray-900">
	<main class="container mx-auto px-4 py-16">
		<!-- Hero Section -->
		{#if mounted}
			<div class="mx-auto mb-20 max-w-3xl text-center" in:fade={{ duration: 800, delay: 300 }}>
				<h1
					class="mb-6 text-4xl font-bold text-gray-900 transition-colors duration-300 md:text-6xl dark:text-white"
				>
					Send Messages. Stay Anonymous.
				</h1>
				<p class="mb-10 text-xl text-gray-600 transition-colors duration-300 dark:text-gray-300">
					A secure platform for anonymous communication, powered by asymmetric cryptography.
				</p>

				<div class="flex flex-col justify-center gap-4 sm:flex-row">
					<button
						onclick={createNewIdentity}
						class="transform rounded-lg bg-indigo-600 px-6 py-3 text-white transition-all duration-300 hover:scale-105 hover:cursor-pointer hover:bg-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 focus:outline-none disabled:opacity-50"
						disabled={isLoading}
					>
						{#if isLoading}
							<span class="inline-flex items-center">
								<span
									class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"
								></span>
								<span>Creating Identity...</span>
							</span>
						{:else}
							<span>Create New Identity</span>
						{/if}
					</button>

					<a
						href="/login"
						class="transform rounded-lg border border-gray-300 px-6 py-3 text-gray-700 transition-all duration-300 hover:scale-105 hover:bg-gray-50 focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 focus:outline-none dark:border-gray-700 dark:text-gray-300 dark:hover:bg-gray-800"
					>
						Access Existing Identity
					</a>
				</div>

				{#if error}
					<p class="mt-4 text-red-600 transition-colors duration-300 dark:text-red-400">{error}</p>
				{/if}
			</div>
		{/if}

		<IdentityModal
			show={showModal}
			isLoading={modalLoading}
			onConfirm={handleConfirm}
			onCancel={handleCancel}
			bind:username={username}
			bind:password={password}
			handleSubmit={handleEnhancedSubmit}
			{form}
		/>

		{#if mounted}
			<div class="mx-auto mb-24 max-w-4xl" transition:fly={{ y: 20, duration: 600 }}>
				<div
					class="rounded-xl bg-white p-6 shadow-lg transition-all duration-300 hover:shadow-xl dark:bg-gray-800"
				>
					<div class="flex flex-col space-y-4">
						<!-- Chat header with animation -->
						<div
							class="flex items-center border-b border-gray-200 pb-3 transition-all duration-300 dark:border-gray-700"
							in:slide={{ duration: 400, delay: 500 }}
						>
							<div
								class="flex h-10 w-10 transform items-center justify-center rounded-full bg-emerald-100 text-emerald-600 transition-colors duration-300 hover:scale-105 dark:bg-emerald-900 dark:text-emerald-400"
							>
								<span class="text-lg font-medium">A</span>
							</div>
							<div class="ml-3">
								<p class="font-medium text-gray-900 dark:text-white">Anonymous Recipient</p>
								<p class="flex items-center text-xs text-gray-500 dark:text-gray-400">
									<span class="mr-1 inline-block h-2 w-2 animate-pulse rounded-full bg-emerald-500"
									></span>
									Online • End-to-end encrypted
								</p>
							</div>

							<!-- Settings button -->
							<button
								class="ml-auto rounded-full p-2 text-gray-400 transition-colors duration-200 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-5 w-5"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
									/>
								</svg>
							</button>
						</div>

						<!-- Chat messages with staggered animations -->
						<div
							class="scrollbar-thin scrollbar-thumb-gray-300 dark:scrollbar-thumb-gray-600 scrollbar-track-transparent max-h-96 space-y-3 overflow-y-auto py-2"
						>
							{#each [{ sender: 'them', text: 'Hi there! I need to share some sensitive information with you.', time: '10:24 AM', delay: 600 }, { sender: 'me', text: 'No problem. Our messages are fully encrypted, so only we can read them.', time: '10:26 AM', delay: 750 }, { sender: 'them', text: "Great. I'll set this message to self-destruct after you read it.", time: '10:27 AM', delay: 900 }, { sender: 'me', text: "Perfect. I'll check it as soon as I can. No one else will be able to see it.", time: '10:28 AM', delay: 1050 }] as message, index (message.time)}
								<div
									class="flex items-end {message.sender === 'me' ? 'justify-end' : ''}"
									in:fade={{ duration: 400, delay: message.delay }}
								>
									<div
										class="{message.sender === 'me'
											? 'bg-emerald-600 text-white'
											: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'}
                         rounded-lg p-3 {message.sender === 'me'
											? 'rounded-br-none'
											: 'rounded-bl-none'}
                         max-w-xs transform shadow-sm transition-all duration-300 hover:scale-[1.01] hover:shadow-md"
									>
										<p>{message.text}</p>
										<p
											class="text-xs {message.sender === 'me'
												? 'text-emerald-200'
												: 'text-gray-500 dark:text-gray-400'} mt-1"
										>
											{message.time}
										</p>
									</div>
								</div>
							{/each}
						</div>

						<!-- New message indicator -->
						<div
							class="animate-pulse py-1 text-center text-xs text-gray-500"
							in:fade={{ duration: 300, delay: 1200 }}
						>
							New messages
						</div>

						<!-- Chat input with animations -->
						<div
							class="mt-3 flex items-center border-t border-gray-200 pt-3 transition-all duration-300 dark:border-gray-700"
							in:slide={{ duration: 400, delay: 1100, axis: 'y' }}
						>
							<button
								class="p-2 text-gray-500 transition-colors duration-200 hover:text-emerald-600 dark:text-gray-400 dark:hover:text-emerald-400"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-5 w-5"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13"
									/>
								</svg>
							</button>

							<input
								type="text"
								placeholder="Type your message..."
								class="mx-2 flex-grow rounded-lg bg-gray-100 p-3 text-gray-900 placeholder-gray-500 transition-all duration-300 focus:ring-2 focus:ring-emerald-500 focus:outline-none dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
							/>

							<button
								class="transform rounded-lg bg-emerald-600 p-3 text-white transition-all duration-300 hover:scale-105 hover:rotate-1 hover:bg-emerald-700 focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:outline-none"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									class="h-5 w-5"
									fill="none"
									viewBox="0 0 24 24"
									stroke="currentColor"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
									/>
								</svg>
							</button>
						</div>
					</div>
				</div>

				<!-- Connection status indicator -->
				<div
					class="mt-2 flex items-center justify-center space-x-1 text-center text-xs text-gray-500 dark:text-gray-400"
					in:fade={{ duration: 300, delay: 1300 }}
				>
					<span class="inline-block h-2 w-2 animate-pulse rounded-full bg-emerald-500"></span>
					<span>Secure connection active</span>
				</div>
			</div>
		{/if}

		<!-- Features Section -->
		{#if mounted}
			<div class="mx-auto mb-24 grid max-w-5xl gap-8 md:grid-cols-3">
				{#each [{ icon: IconName.LOCK, title: 'End-to-End Encryption', description: 'All messages are encrypted with state-of-the-art asymmetric cryptography, ensuring only the intended recipient can read them.' }, { icon: IconName.USERSECRET, title: 'Total Anonymity', description: 'No personally identifiable information is ever collected. Your identity is protected by cryptographic keypairs.' }, { icon: IconName.CHATBUBBLELEFTRIGHT, title: 'Reply Anonymous Messages', description: 'Respond to messages with fun, anonymous replies and keep conversations flowing.' }] as feature, i}
					<div
						class="transform cursor-pointer rounded-lg bg-white p-6 shadow-lg transition-all duration-300 hover:-translate-y-1 hover:shadow-xl dark:bg-gray-800"
						in:fly={{ y: 20, duration: 800, delay: 600 + i * staggerDelay }}
					>
						<div class="mb-4 text-2xl text-indigo-600 dark:text-indigo-400">
							<Icon
								name={feature.icon}
								size={24}
								color={feature.icon === IconName.USERSECRET && $darkMode ? '#7c86ff' : '#4f39f6'}
							/>
						</div>
						<h3
							class="mb-2 text-xl font-semibold text-gray-900 transition-colors duration-300 dark:text-white"
						>
							{feature.title}
						</h3>
						<p class="text-gray-600 transition-colors duration-300 dark:text-gray-300">
							{feature.description}
						</p>
					</div>
				{/each}
			</div>
		{/if}

		<!-- How It Works Section -->
		{#if mounted}
			<div class="mx-auto my-24 max-w-4xl" in:fade={{ duration: 800, delay: 900 }}>
				<h2
					class="mb-10 text-center text-3xl font-bold text-gray-900 transition-colors duration-300 dark:text-white"
				>
					How It Works
				</h2>

				<div class="space-y-12">
					{#each [{ title: 'Create Your Identity', description: 'Generate a unique cryptographic keypair that serves as your anonymous identity on the platform.' }, { title: 'Share Your Public Address', description: 'Share your public key with others so they can send you encrypted messages that only you can decrypt.' }, { title: 'Communicate Securely', description: 'Send and receive encrypted messages without revealing your identity. Only the recipient with the correct private key can decrypt and read your messages.' }] as step, i}
						<div
							class="flex transform flex-col items-center gap-6 transition-all duration-500 hover:scale-102 md:flex-row"
							in:fly={{ x: i % 2 === 0 ? -20 : 20, duration: 800, delay: 1000 + i * staggerDelay }}
						>
							<div class="flex justify-center md:w-1/3">
								<div
									class="flex h-16 w-16 items-center justify-center rounded-full bg-indigo-600 text-2xl font-bold text-white dark:bg-indigo-700"
								>
									{i + 1}
								</div>
							</div>
							<div class="md:w-2/3">
								<h3
									class="mb-2 text-xl font-semibold text-gray-900 transition-colors duration-300 dark:text-white"
								>
									{step.title}
								</h3>
								<p class="text-gray-600 transition-colors duration-300 dark:text-gray-300">
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
			<div class="mx-auto my-24 max-w-3xl text-center" in:fade={{ duration: 800, delay: 1200 }}>
				<h2
					class="mb-6 text-3xl font-bold text-gray-900 transition-colors duration-300 dark:text-white"
				>
					Ready to Communicate Anonymously?
				</h2>
				<p class="mb-10 text-xl text-gray-600 transition-colors duration-300 dark:text-gray-300">
					Join thousands of users who trust Piree for secure, anonymous communication.
				</p>
				<button
					onclick={createNewIdentity}
					class="transform rounded-lg bg-indigo-600 px-6 py-3 text-white transition-all duration-300 hover:scale-105 hover:cursor-pointer hover:bg-white hover:text-indigo-700 hover:outline-2 hover:outline-indigo-700 focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50"
					disabled={isLoading}
				>
					{#if isLoading}
						<span class="mr-2 inline-block animate-spin">⟳</span>
						Creating Identity...
					{:else}
						Get Started Now
					{/if}
				</button>
			</div>
		{/if}
	</main>
</div>
