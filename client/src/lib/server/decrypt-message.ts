import forge from 'node-forge';

export async function decryptMessage(
  encryptedData: string,
  rsaPrivateKey: forge.pki.rsa.PrivateKey
): Promise<string> {
  try {
    const data = forge.util.decode64(encryptedData);
    const buffer = forge.util.createBuffer(data);

    // 2048-bit RSA key (256 bytes)
    const encryptedAesKey = buffer.getBytes(256);
    const iv = buffer.getBytes(12);
	const tag = buffer.getBytes(16);
    const ciphertext = buffer.getBytes();

    let aesKey: string;
    try {
      aesKey = rsaPrivateKey.decrypt(encryptedAesKey, 'RSA-OAEP');
    } catch (err) {
      throw new Error('Failed to decrypt AES key with RSA: ' + (err as Error).message);
    }

    const decipher = forge.cipher.createDecipher('AES-GCM', aesKey);
    decipher.start({ 
		iv: iv,
		tag: forge.util.createBuffer(tag),
		tagLength: 128
	});
    decipher.update(forge.util.createBuffer(ciphertext));
    const success = decipher.finish();

    if (!success) {
      throw new Error('AES-GCM decryption failed: authentication tag mismatch or corrupted data.');
    }

    return forge.util.decodeUtf8(decipher.output.getBytes());
  } catch (err) {
    throw new Error('Decryption error: ' + (err as Error).message);
  }
}
