import forge from 'node-forge';

export async function encryptMessage(message: string, rsaPublicKey: forge.pki.rsa.PublicKey) {
	try {
		const aesKey = forge.random.getBytesSync(16);
		const iv = forge.random.getBytesSync(12);

		const cipher = forge.cipher.createCipher('AES-GCM', aesKey);
		cipher.start({ iv: iv });
		cipher.update(forge.util.createBuffer(forge.util.encodeUtf8(message)));
		cipher.finish();

		const ciphertext = cipher.output.getBytes();
		const tag = cipher.mode.tag.getBytes(); // Get the GCM tag

		const encryptedAesKey = rsaPublicKey.encrypt(aesKey, 'RSA-OAEP');

		const combined = forge.util.createBuffer();
		combined.putBytes(encryptedAesKey);
		combined.putBytes(iv);
		combined.putBytes(tag);
		combined.putBytes(ciphertext);

		return forge.util.encode64(combined.getBytes());
	} catch (err) {
		throw err;
	}
}
