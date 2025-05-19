import { describe, it, expect, beforeAll } from 'vitest';
import forge from 'node-forge';
import { decryptMessage } from './decrypt-message';
import { encryptMessage } from './encrypt-message';

describe('decryptMessage', () => {
  let privateKey: forge.pki.rsa.PrivateKey;
  let publicKey: forge.pki.rsa.PublicKey;
  let wrongKeyPair: forge.pki.KeyPair;
  
  beforeAll(async () => {
    const keyPair = await new Promise<forge.pki.rsa.KeyPair>((resolve, reject) => {
      forge.pki.rsa.generateKeyPair({ bits: 2048, e: 0x10001 }, (err, keypair) => {
        if (err) reject(err);
        else resolve(keypair);
      });
    });
    
    privateKey = keyPair.privateKey;
    publicKey = keyPair.publicKey;

    wrongKeyPair = await new Promise<forge.pki.rsa.KeyPair>((resolve, reject) => {
      forge.pki.rsa.generateKeyPair({ bits: 2048, e: 0x10001 }, (err, keypair) => {
        if (err) reject(err);
        else resolve(keypair);
      });
    });
    
  });

  it('should decrypt encrypted messages correctly', async () => {
    const testMessage = 'Hello, world!';
    const encrypted = await encryptMessage(testMessage, publicKey);
    const decrypted = await decryptMessage(encrypted, privateKey);
    
    expect(decrypted).toBe(testMessage);
  });

  it('should handle empty messages', async () => {
    const emptyMessage = '';
    const encrypted = await encryptMessage(emptyMessage, publicKey);
    const decrypted = await decryptMessage(encrypted, privateKey);
    
    expect(decrypted).toBe(emptyMessage);
  });

  it('should handle Unicode characters', async () => {
    const unicodeMessage = '你好, こんにちは, مرحبا, Привет, αβγδε';
    const encrypted = await encryptMessage(unicodeMessage, publicKey);
    const decrypted = await decryptMessage(encrypted, privateKey);
    
    expect(decrypted).toBe(unicodeMessage);
  });

  it('should handle special characters', async () => {
    const specialCharsMessage = '!@#$%^&*()_+{}|:"<>?~`-=[]\\;\',./';
    const encrypted = await encryptMessage(specialCharsMessage, publicKey);
    const decrypted = await decryptMessage(encrypted, privateKey);
    
    expect(decrypted).toBe(specialCharsMessage);
  });

  it('should handle long messages', async () => {
    const longMessage = 'A'.repeat(10000);
    const encrypted = await encryptMessage(longMessage, publicKey);
    const decrypted = await decryptMessage(encrypted, privateKey);
    
    expect(decrypted).toBe(longMessage);
  });

  it('should throw error for invalid base64', async () => {
    const invalidBase64 = 'not$valid*base64!';
    
    await expect(decryptMessage(invalidBase64, privateKey)).rejects.toThrow('Decryption error:');
  });

  it('should throw error for tampered encrypted data', async () => {
    const message = 'Test tampered message';
    let encrypted = await encryptMessage(message, publicKey);
    
    encrypted = encrypted.substring(0, encrypted.length - 10) + 'AAAA' + encrypted.substring(encrypted.length - 6);
    
    await expect(decryptMessage(encrypted, privateKey)).rejects.toThrow();
  });

  it('should throw error for wrong private key', async () => {
    const message = 'Wrong key test';
    const encrypted = await encryptMessage(message, publicKey);
    
    await expect(decryptMessage(encrypted, wrongKeyPair.privateKey)).rejects.toThrow();
  });

  it('should throw error for truncated data', async () => {
    const message = 'Truncated data test';
    let encrypted = await encryptMessage(message, publicKey);
    
    encrypted = encrypted.substring(0, Math.floor(encrypted.length / 2));
    
    await expect(decryptMessage(encrypted, privateKey)).rejects.toThrow();
  });

  it('should throw error for invalid private key', async () => {
    const message = 'Invalid key test';
    const encrypted = await encryptMessage(message, publicKey);
    const invalidKey = {} as forge.pki.rsa.PrivateKey;
    
    await expect(decryptMessage(encrypted, invalidKey)).rejects.toThrow();
  });
});
