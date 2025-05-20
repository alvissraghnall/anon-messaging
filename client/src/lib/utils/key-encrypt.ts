import * as forge from 'node-forge';

export interface EncryptedKeyData {
  encryptedData: string;
  iv: string;
  salt: string;
}

function deriveKey(password: string, salt: string): string {
  return forge.pkcs5.pbkdf2(password, forge.util.decode64(salt), 10000, 32);
}

/*
export function encryptWithPassword(plaintext: string, password: string): EncryptedKeyData {
  const salt = forge.util.encode64(forge.random.getBytesSync(16));
  const ivBytes = forge.random.getBytesSync(16);
  const iv = forge.util.encode64(ivBytes);

  const key = deriveKey(password, salt);

  const cipher = forge.cipher.createCipher('AES-CBC', key);
  cipher.start({ iv: ivBytes });
  cipher.update(forge.util.createBuffer(plaintext));
  cipher.finish();

  const tag = cipher.mode.tag.getBytes();
  const encryptedData = forge.util.encode64(cipher.output.getBytes());

  return {
    encryptedData,
    iv,
    salt,
    tag: 
  };
}
*/

export function encryptWithPassword(plaintext: string, password: string): string {
  const saltBytes = forge.random.getBytesSync(16);
  const salt = forge.util.encode64(saltBytes);

  const ivBytes = forge.random.getBytesSync(16);

  const key = deriveKey(password, salt);

  const cipher = forge.cipher.createCipher('AES-CBC', key);
  cipher.start({ iv: ivBytes });
  cipher.update(forge.util.createBuffer(plaintext));
  cipher.finish();

  const ciphertext = cipher.output.getBytes();

  const combined = forge.util.createBuffer();
  combined.putBytes(saltBytes);
  combined.putBytes(ivBytes);
  combined.putBytes(ciphertext);

  return forge.util.encode64(combined.getBytes());
}  

/*
export function decryptWithPassword(encrypted: EncryptedKeyData, password: string): string {
  const key = deriveKey(password, encrypted.salt);
  const ivBytes = forge.util.decode64(encrypted.iv);
  const encryptedBytes = forge.util.decode64(encrypted.encryptedData);

  const decipher = forge.cipher.createDecipher('AES-CBC', key);
  decipher.start({ iv: ivBytes });
  decipher.update(forge.util.createBuffer(encryptedBytes));
  const success = decipher.finish();

  if (!success) throw new Error('Decryption failed. Possibly wrong password.');

  return decipher.output.toString();
}
*/

export function decryptWithPassword(encodedData: string, password: string): string {
  const combinedBytes = forge.util.decode64(encodedData);
  const buffer = forge.util.createBuffer(combinedBytes);

  const saltBytes = buffer.getBytes(16); // First 16 bytes: salt
  const ivBytes = buffer.getBytes(16);   // Next 16 bytes: IV
  const ciphertext = buffer.getBytes();  // Remaining bytes: Ciphertext

  const salt = forge.util.encode64(saltBytes);
  const key = deriveKey(password, salt);

  const decipher = forge.cipher.createDecipher('AES-CBC', key);
  decipher.start({ iv: ivBytes });
  decipher.update(forge.util.createBuffer(ciphertext));
  const success = decipher.finish();

  if (!success) {
    throw new Error('Decryption failed. Possibly due to incorrect password or corrupted data.');
  }

  return decipher.output.toString();
}
