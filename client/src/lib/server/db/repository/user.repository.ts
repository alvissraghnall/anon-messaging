import { Kysely } from 'kysely';
import { type Database } from '../db-types';
import bcrypt from 'bcrypt';

export class UserRepository {
	constructor(private db: Kysely<Database>) {}

	async createUser(id: string, username: string, password: string) {
		const hashedPassword = await bcrypt.hash(password, 10);
		await this.db.insertInto('users').values({ id, username, password: hashedPassword }).execute();
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

	async validatePassword(username: string, password: string): Promise<boolean> {
		const user = await this.getUserByUsername(username);
		if (!user) return false;
		return bcrypt.compare(password, user.password);
	}
}
