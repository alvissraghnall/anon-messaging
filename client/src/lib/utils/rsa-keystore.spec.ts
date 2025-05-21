import { describe, it, beforeEach, expect } from 'vitest';
import 'fake-indexeddb/auto';
import { generateRSAKeyPair } from './rsa-keygen';
import { db } from './rsa-keystore';
import * as forge from 'node-forge';
import { encryptWithPassword } from './key-encrypt';

describe('KeyPairDatabase', () => {
	beforeEach(async () => {
		await db.keyPair.clear();
	});

	it('should add and retrieve a KeyPair', async () => {
		const keypair = await generateRSAKeyPair();

		const publicKeyPem = forge.pki.publicKeyToPem(keypair.publicKey);
		const privateKeyPem = forge.pki.privateKeyToPem(keypair.privateKey);

		const username = 'alice';
		const pwd = '0x80201';

		let encPrivKey = encryptWithPassword(privateKeyPem, pwd);
		let encPubKey = encryptWithPassword(publicKeyPem, pwd);

		const id = await db.keyPair.add({
			privateKey: encPrivKey,
			publicKey: encPubKey,
			username
		});

		const stored = await db.keyPair.get(id);

		expect(stored).toBeDefined();
		expect(stored?.username).toBe(username);
		expect(stored?.publicKey).toBe(encPubKey);
		expect(stored?.privateKey).toBe(encPrivKey);
	});

	it('should store multiple KeyPairs', async () => {
		const usernames = ['alice', 'bob'];
		const ids: number[] = [];

		for (const username of usernames) {
			const kp = await generateRSAKeyPair();
			const pub = forge.pki.publicKeyToPem(kp.publicKey);
			const priv = forge.pki.privateKeyToPem(kp.privateKey);

			const pwd = '0x80201' + String.fromCharCode(Math.random() * 10 + 65, Math.random() * 10 + 65);

			console.log(pwd);

			let encPrivKey = encryptWithPassword(priv, pwd);
			let encPubKey = encryptWithPassword(pub, pwd);

			const id = await db.keyPair.add({
				privateKey: encPrivKey,
				publicKey: encPubKey,
				username
			});
			ids.push(id);
		}

		const all = await db.keyPair.toArray();
		expect(all.length).toBe(2);
		expect(all.map((k) => k.username)).toEqual(expect.arrayContaining(usernames));
	});
});
