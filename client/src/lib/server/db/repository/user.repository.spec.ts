import { Kysely, SqliteDialect } from 'kysely';
import Database from "better-sqlite3";
import { afterAll, beforeAll, beforeEach, describe, expect, test } from 'vitest';
import { UserRepository } from './user.repository';
import { type Database as DBTypes } from '../db-types';

let db: Kysely<DBTypes>;
let repo: UserRepository;

beforeAll(async () => {
  db = new Kysely<DBTypes>({
    dialect: new SqliteDialect({
      database: new Database(':memory:'),
    }),
  });

  await db.schema
    .createTable('users')
    .addColumn('id', 'text', col => col.primaryKey())
    .addColumn('username', 'text', col => col.notNull())
    .addColumn('password', 'text', col => col.notNull())
    .execute();

  repo = new UserRepository(db);
});

beforeEach(async () => {
  await db.deleteFrom('users').execute();
});

afterAll(async () => {
  await db.destroy();
});

describe('UserRepository', () => {
  test('create user and retrieve by id', async () => {
    const user = {
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    };

    await repo.createUser(user.id, user.username, user.password);

    const result = await repo.getUserById('user-123');

    expect(result).toEqual({
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    });
  });

  test('retrieve user by username', async () => {
    const user = {
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    };

    await repo.createUser(user.id, user.username, user.password);

    const result = await repo.getUserByUsername('john_doe');

    expect(result).toEqual({
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    });
  });

  test('get user by non-existent id returns undefined', async () => {
    const result = await repo.getUserById('non-existent-id');
    expect(result).toBeUndefined();
  });

  test('get user by non-existent username returns undefined', async () => {
    const result = await repo.getUserByUsername('non-existent-username');
    expect(result).toBeUndefined();
  });

  test('delete user by id', async () => {
    const user = {
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    };

    await repo.createUser(user.id, user.username, user.password);
    
    await repo.deleteUserById('user-123');

    const result = await repo.getUserById('user-123');
    expect(result).toBeUndefined();
  });

  test('delete user by non-existent id does not throw an error', async () => {
    await expect(repo.deleteUserById('non-existent-id')).resolves.not.toThrow();
  });

  test('create multiple users', async () => {
    const user1 = {
      id: 'user-123',
      username: 'john_doe',
      password: 'password123',
    };
    const user2 = {
      id: 'user-124',
      username: 'jane_doe',
      password: 'password456',
    };

    await repo.createUser(user1.id, user1.username, user1.password);
    await repo.createUser(user2.id, user2.username, user2.password);

    const result1 = await repo.getUserById('user-123');
    const result2 = await repo.getUserById('user-124');

    expect(result1).toEqual(user1);
    expect(result2).toEqual(user2);
  });
});
