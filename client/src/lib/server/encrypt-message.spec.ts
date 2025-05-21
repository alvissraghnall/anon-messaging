import { describe, it, expect, beforeAll } from 'vitest';
import forge from 'node-forge';
import { encryptMessage } from './encrypt-message';

describe('encryptMessage', () => {
	let privateKey: forge.pki.rsa.PrivateKey;
	let publicKey: forge.pki.rsa.PublicKey;

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

	it('should return a non-empty base64 string', async () => {
		const message = 'Hello, world!';
		const encrypted = await encryptMessage(message, publicKey);

		expect(encrypted).toBeTypeOf('string');
		expect(encrypted.length).toBeGreaterThan(0);

		expect(() => forge.util.decode64(encrypted)).not.toThrow();
	});

	it('should produce different ciphertexts for the same message on multiple calls', async () => {
		const message = 'Same message test';

		const encrypted1 = await encryptMessage(message, publicKey);
		const encrypted2 = await encryptMessage(message, publicKey);

		expect(encrypted1).not.toBe(encrypted2);
	});

	it('should encrypt messages of different lengths', async () => {
		const messages = [
			'',
			'Short',
			'A medium length message to encrypt',
			"A longer message with multiple sentences. This should also work well with the encryption system. Let's make sure it handles longer content appropriately.",
			'A'.repeat(1000)
		];

		for (const message of messages) {
			const encrypted = await encryptMessage(message, publicKey);
			expect(encrypted).toBeTypeOf('string');
			expect(encrypted.length).toBeGreaterThan(0);
		}
	});

	it('should encrypt messages with special characters', async () => {
		const messages = [
			'Special !@#$%^&*()_+{}|:"<>?~`-=[]\\;\',./characters',
			'Unicode: 你好, こんにちは, مرحبا, Привет, αβγδε',
			'Emojis: 😀 🚀 🌍 🎉 🔒'
		];

		for (const message of messages) {
			const encrypted = await encryptMessage(message, publicKey);
			expect(encrypted).toBeTypeOf('string');
			expect(encrypted.length).toBeGreaterThan(0);
		}
	});

	it('should throw an error if an invalid public key is provided', async () => {
		const message = 'Test message';
		const invalidKey = {} as forge.pki.rsa.PublicKey;

		await expect(encryptMessage(message, invalidKey)).rejects.toThrow();
	});

	it('should correctly structure the encrypted data', async () => {
		const message = 'Test message structure';
		const encrypted = await encryptMessage(message, publicKey);

		const decodedBytes = forge.util.decode64(encrypted);
		const buffer = forge.util.createBuffer(decodedBytes, 'raw');

		const encryptedAesKey = buffer.getBytes(256);
		expect(encryptedAesKey.length).toBe(256);

		const iv = buffer.getBytes(12);
		expect(iv.length).toBe(12);

		const tag = buffer.getBytes(16);
		expect(tag.length).toBe(16);

		const ciphertext = buffer.getBytes();
		expect(ciphertext.length).toBeGreaterThan(0);

		try {
			const decryptedAesKey = privateKey.decrypt(encryptedAesKey, 'RSA-OAEP');
			expect(decryptedAesKey.length).toBe(16); // AES-128 key length

			const decipher = forge.cipher.createDecipher('AES-GCM', decryptedAesKey);
			decipher.start({
				iv: iv,
				tag: forge.util.createBuffer(tag),
				tagLength: 128
			});
			decipher.update(forge.util.createBuffer(ciphertext));
			const result = decipher.finish();

			expect(result).toBe(true);
			expect(decipher.output.toString()).toBe(message);
		} catch (error) {
			throw new Error(`Failed to decrypt: ${error}`);
		}
	});

	it('should handle very long messages', async () => {
		const longMessage = 'A'.repeat(100 * 1024);

		const encrypted = await encryptMessage(longMessage, publicKey);
		expect(encrypted).toBeTypeOf('string');

		const decodedBytes = forge.util.decode64(encrypted);
		const buffer = forge.util.createBuffer(decodedBytes, 'raw');

		const encryptedAesKey = buffer.getBytes(256);
		const iv = buffer.getBytes(12);

		const tag = buffer.getBytes(16);

		const ciphertext = buffer.getBytes();

		const decryptedAesKey = privateKey.decrypt(encryptedAesKey, 'RSA-OAEP');

		const decipher = forge.cipher.createDecipher('AES-GCM', decryptedAesKey);
		decipher.start({
			iv: iv,
			tag: forge.util.createBuffer(tag),
			tagLength: 128
		});
		decipher.update(forge.util.createBuffer(ciphertext));
		const result = decipher.finish();

		expect(result).toBe(true);
		expect(decipher.output.toString()).toBe(longMessage);
	});

	it('should encrypt binary data correctly', async () => {
		let binaryMessage = '';
		for (let i = 0; i < 256; i++) {
			binaryMessage += String.fromCharCode(i);
		}

		const encrypted = await encryptMessage(binaryMessage, publicKey);

		const decodedBytes = forge.util.decode64(encrypted);
		const buffer = forge.util.createBuffer(decodedBytes);

		const encryptedAesKey = buffer.getBytes(256);
		const iv = buffer.getBytes(12);

		const tag = buffer.getBytes(16);
		expect(tag.length).toBe(16);

		const ciphertext = buffer.getBytes();

		const decryptedAesKey = privateKey.decrypt(encryptedAesKey, 'RSA-OAEP');

		const decipher = forge.cipher.createDecipher('AES-GCM', decryptedAesKey);
		decipher.start({
			iv: iv,
			tag: forge.util.createBuffer(tag),
			tagLength: 128
		});
		decipher.update(forge.util.createBuffer(ciphertext));
		const result = decipher.finish();

		expect(result).toBe(true);
		expect(forge.util.decodeUtf8(decipher.output.getBytes())).toBe(binaryMessage);
	});

	// Helper function to implement full decryption flow for testing
	async function decryptMessage(
		encrypted: string,
		rsaPrivateKey: forge.pki.rsa.PrivateKey
	): Promise<string> {
		const decodedBytes = forge.util.decode64(encrypted);
		const buffer = forge.util.createBuffer(decodedBytes);

		const encryptedAesKey = buffer.getBytes(256);

		const iv = buffer.getBytes(12);

		const tag = buffer.getBytes(16);
		expect(tag.length).toBe(16);

		const ciphertext = buffer.getBytes();

		const aesKey = rsaPrivateKey.decrypt(encryptedAesKey, 'RSA-OAEP');

		const decipher = forge.cipher.createDecipher('AES-GCM', aesKey);
		decipher.start({
			iv: iv,
			tag: forge.util.createBuffer(tag),
			tagLength: 128
		});
		decipher.update(forge.util.createBuffer(ciphertext));
		const result = decipher.finish();

		if (!result) {
			throw new Error('Failed to decrypt message: Auth tag verification failed');
		}

		return forge.util.decodeUtf8(decipher.output.getBytes());
	}

	it('should perform full encryption and decryption roundtrip', async () => {
		const testMessages = [
			'Simple test message',
			'A longer message with some special characters: !@#$%^&*()',
			'Unicode test: 你好, こんにちは, مرحبا',
			'A'.repeat(10000)
		];

		for (const message of testMessages) {
			const encrypted = await encryptMessage(message, publicKey);
			const decrypted = await decryptMessage(encrypted, privateKey);

			expect(decrypted).toBe(message);
		}
	});
});
