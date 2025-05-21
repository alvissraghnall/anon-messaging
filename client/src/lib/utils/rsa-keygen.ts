import forge, { type pki } from 'node-forge';

export async function generateRSAKeyPair(): Promise<pki.KeyPair> {
	return await new Promise((resolve, reject) => {
		forge.pki.rsa.generateKeyPair({ bits: 2048, e: 0x10001 }, (err, keypair) => {
			if (err) reject(err);
			else resolve(keypair);
		});
	});
}
