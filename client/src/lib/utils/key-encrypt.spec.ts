import { describe, it, test, expect } from 'vitest';
import {
  encryptWithPassword,
  decryptWithPassword,
  type EncryptedKeyData
} from './key-encrypt';
import * as forge from 'node-forge';

describe('Encryption/Decryption with Password', () => {
  const plaintext = '-----BEGIN TEST KEY-----\nFAKEKEYDATA\n-----END TEST KEY-----';
  const password = 'strongpassword123';

  it('should encrypt and decrypt back to original text', () => {
    const encrypted = encryptWithPassword(plaintext, password);
    expect(encrypted).toBeDefined();

    expect(encrypted).toBeTypeOf('string');
    expect(encrypted.length).toBeGreaterThan(0);

    expect(() => forge.util.decode64(encrypted)).not.toThrow();
    
    const decrypted = decryptWithPassword(encrypted, password);
    expect(decrypted).toBe(plaintext);
  });

  it('should fail decryption with wrong password', () => {
    const encrypted = encryptWithPassword(plaintext, password);
    expect(() => {
      decryptWithPassword(encrypted, 'wrongPassword');
    }).toThrow('Decryption failed. Possibly due to incorrect password or corrupted data.');
  });

  it('produces different encrypted data for same input due to random salt/IV', () => {
    const encrypted1 = encryptWithPassword(plaintext, password);
    const encrypted2 = encryptWithPassword(plaintext, password);

    console.log(encrypted1);

    expect(encrypted1).not.toBe(encrypted2);
    expect(encrypted1).toBeTypeOf('string');
    expect(encrypted2).toBeTypeOf('string');
    expect(encrypted2.length).toBeGreaterThan(0);

  });
});
