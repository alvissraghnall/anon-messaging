import { describe, it, expect, beforeAll, vi } from 'vitest';
import { generateRSAKeyPair } from './rsa-keygen';
import forge from 'node-forge';

describe('generateRSAKeyPair', () => {
	let keyPair: forge.pki.KeyPair;

	beforeAll(async () => {
		keyPair = await generateRSAKeyPair();
	});

	it('should return a valid key pair object', () => {
		expect(keyPair).toBeTypeOf('object');
		expect(keyPair.publicKey).toBeDefined();
		expect(keyPair.privateKey).toBeDefined();
	});

	it('should generate a 2048-bit RSA key pair', () => {
		const publicKeyPem = forge.pki.publicKeyToPem(keyPair.publicKey);
		expect(publicKeyPem).toContain('-----BEGIN PUBLIC KEY-----');
		expect(publicKeyPem).toContain('-----END PUBLIC KEY-----');

		const privateKeyPem = forge.pki.privateKeyToPem(keyPair.privateKey);
		expect(privateKeyPem).toContain('-----BEGIN RSA PRIVATE KEY-----');
		expect(privateKeyPem).toContain('-----END RSA PRIVATE KEY-----');

		// Verify key size
		expect(keyPair.privateKey.n.bitLength()).toBe(2048);
	});

	it('should have the correct public exponent (0x10001)', () => {
		const publicExponent = keyPair.publicKey.e;
		expect(publicExponent.toString(16)).toBe('10001');
	});

	it('should generate unique key pairs on multiple calls', async () => {
		const anotherKeyPair = await generateRSAKeyPair();

		const firstPrivatePem = forge.pki.privateKeyToPem(keyPair.privateKey);
		const secondPrivatePem = forge.pki.privateKeyToPem(anotherKeyPair.privateKey);

		expect(firstPrivatePem).not.toBe(secondPrivatePem);
	});

	it('should generate a key pair that can encrypt and decrypt', () => {
		const testMessage = 'This is a test message for encryption/decryption';

		const encrypted = keyPair.publicKey.encrypt(testMessage, 'RSA-OAEP');
		expect(encrypted).toBeTypeOf('string');
		expect(encrypted).not.toBe(testMessage);

		const decrypted = keyPair.privateKey.decrypt(encrypted, 'RSA-OAEP');
		expect(decrypted).toBe(testMessage);
	});

	it('should generate a key pair that can sign and verify', () => {
		const testMessage = 'This is a test message for signing';

		const md = forge.md.sha256.create();
		md.update(testMessage, 'utf8');

		const signature = keyPair.privateKey.sign(md);
		expect(signature).toBeTypeOf('string');

		const verified = keyPair.publicKey.verify(md.digest().bytes(), signature);
		expect(verified).toBe(true);
	});

	it('should reject when an error occurs during key generation', async () => {
		// Mock the forge library to simulate an error
		const originalGenerateKeyPair = forge.pki.rsa.generateKeyPair;
		forge.pki.rsa.generateKeyPair = vi.fn((options, callback) => {
			callback(new Error('Simulated error'));
		});

		await expect(generateRSAKeyPair()).rejects.toThrow('Simulated error');

		forge.pki.rsa.generateKeyPair = originalGenerateKeyPair;
	});
});
