<script lang="ts">
  import { onMount } from 'svelte';
  import type { PageProps } from './$types';

  let { data }: PageProps = $props();
  
  let currentDate = new Date("2025-03-15").toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  });
  
</script>


<div class="dark:bg-gray-900 py-8 pb-10">

	<div class="container mx-auto px-4 py-8 max-w-4xl">
		<h1 class="text-3xl font-bold mb-6 text-gray-900 dark:text-white">Terms of Service</h1>
		<p class="text-sm text-gray-600 dark:text-gray-400 mb-6">Last Updated: {currentDate}</p>

		<div class="prose prose-slate dark:prose-invert max-w-none">
			<p class="text-gray-700 dark:text-gray-300">
				Welcome to our anonymous messaging service. Please read these Terms of Service ("Terms") carefully
				before
				using our platform.
				By accessing or using our service, you agree to be bound by these Terms. If you disagree with any part
				of
				these Terms, you may not access our service.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">1. Service Description</h2>
			<p class="text-gray-700 dark:text-gray-300">
				Our service provides a platform for sending and receiving anonymous messages using cryptographic
				technology.
				We utilize asymmetric keypairs to ensure anonymity between users.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">2. User Responsibilities</h2>
			<p class="text-gray-700 dark:text-gray-300">
				2.1. <strong>Account Security:</strong> You are responsible for safeguarding your cryptographic keys.
				Any
				activity that occurs using your keys is your responsibility.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				2.2. <strong>Legal Compliance:</strong> You agree to use our service only for lawful purposes and in
				accordance with these Terms. You will not use our service for any illegal or unauthorized purpose.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				2.3. <strong>Content Guidelines:</strong> You agree not to transmit any content that:
			</p>
			<ul class="list-disc pl-6 my-3 text-gray-700 dark:text-gray-300">
				<li>Is illegal, harmful, threatening, abusive, harassing, defamatory, or otherwise objectionable</li>
				<li>Encourages criminal conduct or would give rise to civil liability</li>
				<li>Infringes any patent, trademark, trade secret, copyright, or other intellectual property rights</li>
				<li>Contains software viruses or any computer code designed to disrupt or damage our systems</li>
				<li>Constitutes unsolicited or unauthorized advertising or promotional materials</li>
				<li>Involves the transmission of "junk mail," "chain letters," or unsolicited mass mailing</li>
			</ul>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">3. Privacy and Data Collection
			</h2>
			<p class="text-gray-700 dark:text-gray-300">
				3.1. <strong>Anonymity:</strong> Our service is designed to provide anonymity. However, we cannot
				guarantee
				perfect anonymity under all circumstances, particularly in cases of illegal activity where legal
				authorities
				may compel disclosure.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				3.2. <strong>Data Collection:</strong> We collect minimal data necessary to operate our service. This
				may
				include:
			</p>
			<ul class="list-disc pl-6 my-3 text-gray-700 dark:text-gray-300">
				<li>Public keys (not linked to personal identity)</li>
				<li>Message timestamps (not linked to identifiable information)</li>
				<li>Technical metadata necessary for service operation</li>
			</ul>
			<p class="text-gray-700 dark:text-gray-300">
				3.3. <strong>No Personal Data:</strong> We do not intentionally collect personally identifiable
				information.
				However, users should be aware that certain patterns of usage or content shared may potentially be
				identifiable.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">4. Intellectual Property</h2>
			<p class="text-gray-700 dark:text-gray-300">
				4.1. <strong>Our Rights:</strong> The service, including all software, design elements, and content
				created
				by us, is owned by us and protected by intellectual property laws.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				4.2. <strong>Your Content:</strong> You retain ownership of any content you transmit through our
				service. By
				submitting content, you grant us a non-exclusive, worldwide, royalty-free license to use, reproduce, and
				distribute your content solely for the purpose of providing our service.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">5. Limitation of Liability</h2>
			<p class="text-gray-700 dark:text-gray-300">
				5.1. <strong>Service Provided "As Is":</strong> Our service is provided on an "as is" and "as available"
				basis without warranties of any kind, either express or implied.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				5.2. <strong>No Guarantee of Availability:</strong> We do not guarantee that our service will be
				uninterrupted, timely, secure, or error-free.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				5.3. <strong>Limitation of Damages:</strong> In no event shall we be liable for any direct, indirect,
				incidental, special, or consequential damages arising out of or in any way connected with the use of our
				service.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">6. Termination</h2>
			<p class="text-gray-700 dark:text-gray-300">
				6.1. <strong>At Our Discretion:</strong> We may terminate or suspend your access to our service
				immediately,
				without prior notice or liability, for any reason, including if you breach these Terms.
			</p>
			<p class="text-gray-700 dark:text-gray-300">
				6.2. <strong>Effect of Termination:</strong> Upon termination, your right to use the service will
				immediately cease. All provisions of these Terms which by their nature should survive termination shall
				survive.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">7. Prohibited Uses</h2>
			<p class="text-gray-700 dark:text-gray-300">You are expressly prohibited from using our service for:</p>
			<ul class="list-disc pl-6 my-3 text-gray-700 dark:text-gray-300">
				<li>Illegal activities including but not limited to fraud, terrorism, harassment, or distribution of
					illegal
					content</li>
				<li>Unauthorized access to other computer systems</li>
				<li>Distribution of malware, viruses, or other malicious code</li>
				<li>Interference with other users' enjoyment of the service</li>
				<li>Impersonation of any person or entity</li>
				<li>Commercial solicitation or spam</li>
			</ul>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">8. Indemnification</h2>
			<p class="text-gray-700 dark:text-gray-300">
				You agree to indemnify, defend, and hold harmless our company, its officers, directors, employees, and
				agents from and against any claims, liabilities, damages, losses, and expenses, including without
				limitation
				reasonable attorney's fees, arising out of or in any way connected with your access to or use of our
				service
				or your violation of these Terms.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">9. Governing Law</h2>
			<p class="text-gray-700 dark:text-gray-300">
				These Terms shall be governed by and construed in accordance with the laws of {data.jurisdiction},
				without
				regard to its conflict of law provisions.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">10. Changes to Terms</h2>
			<p class="text-gray-700 dark:text-gray-300">
				We reserve the right to modify or replace these Terms at any time. It is your responsibility to review
				these
				Terms periodically for changes. Your continued use of our service following the posting of any changes
				constitutes acceptance of those changes.
			</p>

			<h2 class="text-xl font-semibold mt-6 mb-3 text-gray-900 dark:text-white">11. Contact Information</h2>
			<p class="text-gray-700 dark:text-gray-300">
				For questions about these Terms, please contact us at {data.email}.
			</p>
		</div>
	</div>
</div>