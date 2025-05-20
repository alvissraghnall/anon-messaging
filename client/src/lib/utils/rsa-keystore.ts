import Dexie, { type EntityTable } from 'dexie';
import type { URLSafeBase64 } from './base64-to-url-safe';

interface KeyPair {
  id: number;
  privateKey: URLSafeBase64,
  publicKey: URLSafeBase64,
  username: string
}

export const db = new Dexie('KeyPairDatabase') as Dexie & {
  keyPair: EntityTable<
    KeyPair,
    'id'
  >;
};

// Schema declaration:
db.version(1).stores({
  keyPair: '++id, privateKey, publicKey, username' // primary key "id" (for the runtime!)
});

export type { KeyPair };

