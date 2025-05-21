<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	let currentDate = new Date('2025-03-15').toLocaleDateString('en-US', {
		year: 'numeric',
		month: 'long',
		day: 'numeric'
	});
</script>

<svelte:head>
	<title>Privacy Policy | Anonymous Messaging Service</title>
	<meta name="description" content="Privacy Policy for our anonymous messaging service" />
</svelte:head>

<div
	class="min-h-screen bg-gray-50 text-gray-900 transition-colors duration-200 dark:bg-gray-900 dark:text-gray-100"
>
	<div class="container mx-auto max-w-4xl px-4 py-8">
		<div class="mb-8 flex items-center justify-between">
			<h1 class="text-3xl font-bold">Privacy Policy</h1>
		</div>

		<div class="prose dark:prose-invert max-w-none">
			<p class="text-lg">Last Updated: {currentDate}</p>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Introduction</h2>
				<p>
					Welcome to our anonymous messaging service. We are committed to protecting your privacy
					and ensuring the security of your information. This Privacy Policy explains how we
					collect, use, and safeguard your information when you use our service.
				</p>
				<p>
					By using our service, you agree to the collection and use of information in accordance
					with this policy. Our service is designed with privacy and anonymity as core principles.
				</p>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Information We Don't Collect</h2>
				<p>Our service is designed to provide anonymity. We do not collect:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Personal identification information</li>
					<li>Names or email addresses</li>
					<li>Social media profiles</li>
					<li>Physical addresses</li>
					<li>Phone numbers</li>
					<li>Message content (all messages are end-to-end encrypted)</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Information We Do Collect</h2>
				<p>We collect minimal information necessary to provide our service:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Public keys (generated and stored client-side)</li>
					<li>Encrypted message data (which we cannot decrypt)</li>
					<li>Message timestamps (for operational purposes only)</li>
					<li>
						Technical information such as:
						<ul class="mt-2 list-disc pl-6">
							<li>IP addresses (temporarily, for security and abuse prevention)</li>
							<li>Browser type and version</li>
							<li>Operating system information</li>
							<li>Access times and dates</li>
						</ul>
					</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Cryptography and Security</h2>
				<p>Our service uses asymmetric key cryptography implemented in a Rust microservice:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>All communications are secured using cryptographic keypairs</li>
					<li>Private keys are generated and stored only on your device</li>
					<li>We have no access to your private keys</li>
					<li>We cannot decrypt or read any messages sent through our service</li>
					<li>Messages are encrypted end-to-end</li>
				</ul>
				<p>
					While we implement industry-standard security measures, no method of transmission over the
					Internet or electronic storage is 100% secure. We cannot guarantee absolute security.
				</p>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">How We Use Information</h2>
				<p>The limited information we collect is used only for:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Providing and maintaining our service.</li>
					<li>Improving our service.</li>
					<li>Monitoring for abuse and security purposes.</li>
					<li>Ensuring operational functionality.</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Data Retention</h2>
				<p>
					We retain the minimal data we collect for as long as necessary to provide our service:
				</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Encrypted messages are stored until they are deleted by users</li>
					<li>IP addresses are stored temporarily for security monitoring</li>
					<li>Technical logs are automatically deleted after 30 days</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Cookies and Local Storage</h2>
				<p>Our service uses minimal cookies and local storage:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Local storage is used to store your cryptographic keys and theme preferences.</li>
					<li>We do not use tracking or advertising cookies.</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Third-Party Services</h2>
				<p>
					Our service does not integrate with third-party services that would compromise anonymity.
					We do not:
				</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Share any data with third parties.</li>
					<li>Use third-party analytics.</li>
					<li>Integrate with social media platforms.</li>
					<li>Use advertising networks.</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Legal Compliance</h2>
				<p>
					Despite our commitment to privacy, we may be required to disclose information in response
					to valid legal requests such as court orders or subpoenas. However, due to the nature of
					our service:
				</p>
				<ul class="mb-4 list-disc pl-6">
					<li>We cannot decrypt message content</li>
					<li>We do not have access to user identities</li>
					<li>We have minimal information to provide</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">User Responsibilities</h2>
				<p>While our service provides technical anonymity, users must:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Not share identifying information in messages</li>
					<li>Protect their private keys</li>
					<li>Use the service responsibly and legally</li>
					<li>Consider using additional privacy protections like VPNs if needed</li>
				</ul>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Changes to This Privacy Policy</h2>
				<p>
					We may update our Privacy Policy from time to time. We will notify you of any changes by
					posting the new Privacy Policy on this page and updating the "Last Updated" date.
				</p>
				<p>
					You are advised to review this Privacy Policy periodically for any changes. Changes to
					this Privacy Policy are effective when they are posted on this page.
				</p>
			</section>

			<section class="my-8">
				<h2 class="mb-4 text-2xl font-semibold">Contact Us</h2>
				<p>If you have any questions about this Privacy Policy, please contact us at:</p>
				<ul class="mb-4 list-disc pl-6">
					<li>Through our anonymous feedback form on {data.pageUrl}/contact</li>
					<li>By email: {data.email}</li>
				</ul>
			</section>
		</div>
	</div>
</div>
