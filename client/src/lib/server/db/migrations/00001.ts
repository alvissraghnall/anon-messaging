import { Kysely, sql } from 'kysely'

export async function up(db: Kysely<any>): Promise<void> {

  await db.schema
    .createTable('users')
    .ifNotExists()
    .addColumn('id', 'text', col => col.primaryKey())
    .addColumn('username', 'text', col => col.notNull())
    .addColumn('password', 'text', col => col.notNull())
    .execute()

  
  await db.schema
    .createTable('anon_mappings')
    .ifNotExists()
    .addColumn('anon_id', 'text', col => col.primaryKey())
    .addColumn('user_id', 'text', col => col.notNull())
    .addColumn('created_at', 'integer', col => col.notNull())
    .addColumn('expires_at', 'integer', col => col.notNull())
    .addColumn('thread_id', 'integer', col => col.notNull())
    .addForeignKeyConstraint('fk_anon_user', ['user_id'], 'users', ['id'])
    .execute()


  await db.schema
    .createIndex('idx_anon_mappings_user')
    .on('anon_mappings')
    .ifNotExists()
    .columns(['user_id', 'expires_at'])
    .execute()

  await db.schema
    .createIndex('idx_anon_mappings_thread')
    .on('anon_mappings')
    .ifNotExists()
    .columns(['thread_id'])
    .execute()
}

export async function down(db: Kysely<any>): Promise<void> {

  await db.schema.dropIndex('idx_anon_mappings_thread').ifExists().execute()
  await db.schema.dropIndex('idx_anon_mappings_user').ifExists().execute()


  await db.schema.dropTable('anon_mappings').ifExists().execute()
  await db.schema.dropTable('users').ifExists().execute()
}
