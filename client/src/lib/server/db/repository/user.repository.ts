import { Kysely } from 'kysely';
import { type Database } from '../db-types';

export class UserRepository {
	constructor(private db: Kysely<Database>) {}

	async createUser(id: string, username: string, password: string) {
		await this.db.insertInto('users').values({ id, username, password }).execute();
	}

	async getUserById(id: string) {
		return this.db.selectFrom('users').selectAll().where('id', '=', id).executeTakeFirst();
	}

	async getUserByUsername(username: string) {
		return this.db
			.selectFrom('users')
			.selectAll()
			.where('username', '=', username)
			.executeTakeFirst();
	}

	async deleteUserById(id: string) {
		await this.db.deleteFrom('users').where('id', '=', id).execute();
	}
}
