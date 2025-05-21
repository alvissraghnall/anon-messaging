import forge from 'node-forge';

export async function signMessage(
	message: string,
	rsaPrivateKey: forge.pki.rsa.PrivateKey
): Promise<string> {
	try {
		const md = forge.md.sha256.create();
		md.update(message, 'utf8');
		const signature = rsaPrivateKey.sign(md);
		return forge.util.encode64(signature);
	} catch (err) {
		throw err;
	}
}

export async function verifySignature(
	message: string,
	signature: string,
	rsaPublicKey: forge.pki.rsa.PublicKey
): Promise<boolean> {
	try {
		const md = forge.md.sha256.create();
		md.update(message, 'utf8');
		const verified = rsaPublicKey.verify(md.digest().bytes(), forge.util.decode64(signature));
		return verified;
	} catch (err) {
		throw err;
	}
}
