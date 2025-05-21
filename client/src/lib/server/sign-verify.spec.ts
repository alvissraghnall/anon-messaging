import { describe, it, expect, beforeAll } from 'vitest';
import forge from 'node-forge';
import { signMessage, verifySignature } from './sign-verify';

describe('RSA Signature Functions', () => {
	// Test keys
	let privateKey: forge.pki.rsa.PrivateKey;
	let publicKey: forge.pki.rsa.PublicKey;

	// Generate RSA key pair once for all tests
	beforeAll(async () => {
		const keyPair = await new Promise<forge.pki.rsa.KeyPair>((resolve, reject) => {
			forge.pki.rsa.generateKeyPair({ bits: 2048, e: 0x10001 }, (err, keypair) => {
				if (err) reject(err);
				else resolve(keypair);
			});
		});

		privateKey = keyPair.privateKey;
		publicKey = keyPair.publicKey;
	});

	describe('signMessage', () => {
		it('should generate a base64 encoded signature', async () => {
			const message = 'Hello, world!';
			const signature = await signMessage(message, privateKey);

			// Check that signature is a non-empty string
			expect(signature).toBeTypeOf('string');
			expect(signature.length).toBeGreaterThan(0);

			// Verify it's valid base64
			expect(() => forge.util.decode64(signature)).not.toThrow();
		});

		it('should generate different signatures for different messages', async () => {
			const message1 = 'First message';
			const message2 = 'Second message';

			const signature1 = await signMessage(message1, privateKey);
			const signature2 = await signMessage(message2, privateKey);

			expect(signature1).not.toBe(signature2);
		});

		it('should generate consistent signatures for the same message', async () => {
			const message = 'Consistent message test';

			const signature1 = await signMessage(message, privateKey);
			const signature2 = await signMessage(message, privateKey);

			expect(signature1).toBe(signature2);
		});

		it('should throw an error if an invalid private key is provided', async () => {
			const message = 'Test message';
			const invalidKey = {} as forge.pki.rsa.PrivateKey;

			await expect(signMessage(message, invalidKey)).rejects.toThrow();
		});
	});

	describe('verifySignature', () => {
		it('should verify a valid signature correctly', async () => {
			const message = 'Test verification message';
			const signature = await signMessage(message, privateKey);

			const isValid = await verifySignature(message, signature, publicKey);
			expect(isValid).toBe(true);
		});

		it('should reject a signature for a different message', async () => {
			const originalMessage = 'Original message';
			const differentMessage = 'Different message';

			const signature = await signMessage(originalMessage, privateKey);
			const isValid = await verifySignature(differentMessage, signature, publicKey);

			expect(isValid).toBe(false);
		});

		it('should reject a tampered signature', async () => {
			const message = 'Tamper test message';
			let signature = await signMessage(message, privateKey);

			// Tamper with the signature by changing a character
			if (signature.length > 0) {
				const charToReplace = signature.charAt(5);
				const replacement = charToReplace === 'A' ? 'B' : 'A';
				signature = signature.substring(0, 5) + replacement + signature.substring(6);
			}

			await expect(verifySignature(message, signature, publicKey)).rejects.toThrow();
		});

		it('should throw an error if an invalid signature is provided', async () => {
			const message = 'Error test message';
			const invalidSignature = 'not_a_valid_base64_!@#$';

			await expect(verifySignature(message, invalidSignature, publicKey)).rejects.toThrow();
		});

		it('should throw an error if an invalid public key is provided', async () => {
			const message = 'Public key test';
			const signature = await signMessage(message, privateKey);
			const invalidKey = {} as forge.pki.rsa.PublicKey;

			await expect(verifySignature(message, signature, invalidKey)).rejects.toThrow();
		});
	});

	describe('End-to-end signature workflow', () => {
		it('should successfully sign and verify across multiple messages', async () => {
			const messages = [
				'Short message',
				'A longer message with some more content to sign',
				'Message with special characters: !@#$%^&*()',
				'Message with numbers: 1234567890',
				'Message with unicode: 你好, こんにちは, مرحبا'
			];

			for (const message of messages) {
				const signature = await signMessage(message, privateKey);
				const isValid = await verifySignature(message, signature, publicKey);
				expect(isValid).toBe(true);
			}
		});

		it('should work with empty messages', async () => {
			const emptyMessage = '';
			const signature = await signMessage(emptyMessage, privateKey);
			const isValid = await verifySignature(emptyMessage, signature, publicKey);

			expect(isValid).toBe(true);
		});

		it('should work with very long messages', async () => {
			// Create a long message (100KB of text)
			const longMessage = 'a'.repeat(100 * 1024);

			const signature = await signMessage(longMessage, privateKey);
			const isValid = await verifySignature(longMessage, signature, publicKey);

			expect(isValid).toBe(true);
		});
	});
});
