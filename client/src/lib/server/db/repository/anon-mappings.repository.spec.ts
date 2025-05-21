import { Kysely, SqliteDialect } from 'kysely';
import Database from 'better-sqlite3';

import { afterAll, beforeAll, beforeEach, describe, expect, test } from 'vitest';
import { AnonMappingRepository } from './anon-mappings.repository';
import { type Database as DBTypes } from '../db-types';

let db: Kysely<DBTypes>;
let repo: AnonMappingRepository;

beforeAll(async () => {
	db = new Kysely<DBTypes>({
		dialect: new SqliteDialect({
			database: new Database(':memory:')
		})
	});

	await db.schema
		.createTable('users')
		.addColumn('id', 'text', (col) => col.primaryKey())
		.addColumn('username', 'text', (col) => col.notNull())
		.addColumn('password', 'text', (col) => col.notNull())
		.execute();

	await db.schema
		.createTable('anon_mappings')
		.ifNotExists()
		.addColumn('anon_id', 'text', (col) => col.primaryKey())
		.addColumn('user_id', 'text', (col) => col.notNull())
		.addColumn('created_at', 'integer', (col) => col.notNull())
		.addColumn('expires_at', 'integer', (col) => col.notNull())
		.addColumn('thread_id', 'integer', (col) => col.notNull())
		.addForeignKeyConstraint('fk_anon_user', ['user_id'], 'users', ['id'])
		.execute();

	repo = new AnonMappingRepository(db);
});

beforeEach(async () => {
	await db.deleteFrom('anon_mappings').execute();
	await db.deleteFrom('users').execute();

	await db
		.insertInto('users')
		.values({ id: 'user-123', username: 'asap', password: 'rocky' })
		.execute();
});

afterAll(async () => {
	await db.destroy();
});

describe('AnonMappingRepository', () => {
	test('create and retrieve mapping', async () => {
		const mapping = {
			anon_id: 'anon-abc',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};

		await repo.createMapping(mapping);

		const result = await repo.getMappingByAnonId('anon-abc');

		expect(result).toEqual(mapping);
	});

	test('get mappings by user ID', async () => {
		const mapping1 = {
			anon_id: 'anon-1',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};
		const mapping2 = {
			anon_id: 'anon-2',
			user_id: 'user-123',
			created_at: 1000001,
			expires_at: 2000001,
			thread_id: 2
		};

		await repo.createMapping(mapping1);
		await repo.createMapping(mapping2);

		const results = await repo.getMappingsByUserId('user-123');

		expect(results).toHaveLength(2);
		expect(results).toContainEqual(mapping1);
		expect(results).toContainEqual(mapping2);
	});

	test('delete expired mappings', async () => {
		const expired = {
			anon_id: 'expired',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 1100000,
			thread_id: 1
		};
		const active = {
			anon_id: 'active',
			user_id: 'user-123',
			created_at: 1000001,
			expires_at: 9999999,
			thread_id: 2
		};

		await repo.createMapping(expired);
		await repo.createMapping(active);

		await repo.deleteExpiredMappings(1200000);

		const remaining = await repo.getMappingsByUserId('user-123');

		expect(remaining).toHaveLength(1);
		expect(remaining[0]).toEqual(active);
	});

	test('get mapping by non-existent anon_id returns undefined', async () => {
		const result = await repo.getMappingByAnonId('non-existent-id');
		expect(result).toBeUndefined();
	});

	test('create mapping with invalid user_id fails', async () => {
		const invalidMapping = {
			anon_id: 'anon-invalid',
			user_id: 'invalid-user',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};

		// Simulate an invalid user ID by trying to insert a mapping with a non-existent user_id
		await expect(repo.createMapping(invalidMapping)).rejects.toThrowError(
			'FOREIGN KEY constraint failed'
		);
	});

	test('delete mapping by anon_id', async () => {
		const mapping = {
			anon_id: 'anon-delete',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};

		await repo.createMapping(mapping);

		await repo.deleteMappingByAnonId('anon-delete');

		const result = await repo.getMappingByAnonId('anon-delete');
		expect(result).toBeUndefined();
	});

	test('attempting to delete mapping that does not exist does not throw an error', async () => {
		await expect(repo.deleteMappingByAnonId('non-existent-anon')).resolves.not.toThrow();
	});

	test('create multiple mappings with the same user_id but different anon_ids', async () => {
		const mapping1 = {
			anon_id: 'anon-1',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};
		const mapping2 = {
			anon_id: 'anon-2',
			user_id: 'user-123',
			created_at: 1000001,
			expires_at: 2000001,
			thread_id: 2
		};

		await repo.createMapping(mapping1);
		await repo.createMapping(mapping2);

		const result = await repo.getMappingsByUserId('user-123');
		expect(result).toHaveLength(2);
		expect(result).toContainEqual(mapping1);
		expect(result).toContainEqual(mapping2);
	});

	test('create mapping with future expiration date', async () => {
		const mapping = {
			anon_id: 'anon-future',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 4000000,
			thread_id: 3
		};

		await repo.createMapping(mapping);

		const result = await repo.getMappingByAnonId('anon-future');
		expect(result).toEqual(mapping);
	});

	test('delete expired mappings at specific time', async () => {
		const mapping1 = {
			anon_id: 'anon-1',
			user_id: 'user-123',
			created_at: 1000000,
			expires_at: 2000000,
			thread_id: 1
		};
		const mapping2 = {
			anon_id: 'anon-2',
			user_id: 'user-123',
			created_at: 1000001,
			expires_at: 999999, // expired mapping
			thread_id: 2
		};

		await repo.createMapping(mapping1);
		await repo.createMapping(mapping2);

		await repo.deleteExpiredMappings(1000001);

		const result = await repo.getMappingsByUserId('user-123');
		expect(result).toHaveLength(1);
		expect(result[0]).toEqual(mapping1);
	});
});
