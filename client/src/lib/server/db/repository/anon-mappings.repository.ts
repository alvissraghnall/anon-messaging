import { Kysely } from 'kysely';
import { type Database } from '../db-types';

export class AnonMappingRepository {
  constructor(private db: Kysely<Database>) {}

  async createMapping(mapping: {
    anon_id: string;
    user_id: string;
    created_at: number;
    expires_at: number;
    thread_id: number;
  }) {
    await this.db.insertInto('anon_mappings')
      .values(mapping)
      .execute();
  }

  async getMappingByAnonId(anon_id: string) {
    return this.db.selectFrom('anon_mappings')
      .selectAll()
      .where('anon_id', '=', anon_id)
      .executeTakeFirst();
  }

  async getMappingsByUserId(user_id: string) {
    return this.db.selectFrom('anon_mappings')
      .selectAll()
      .where('user_id', '=', user_id)
      .execute();
  }

  async deleteExpiredMappings(currentTime: number) {
    await this.db.deleteFrom('anon_mappings')
      .where('expires_at', '<', currentTime)
      .execute();
  }

  async deleteMappingByAnonId(anonId: string) {
    await this.db.deleteFrom('anon_mappings')
      .where('anon_id', '=', anonId)
      .execute();
  }
}
