import { Kysely, SqliteDialect } from 'kysely'
//import { BunSqliteDialect } from 'kysely-bun-sqlite'
import Database from "better-sqlite3";
import path from 'path';
import { type Database as DBTypes } from './db-types';

const dbPath = path.resolve(process.cwd(), 'db.sqlite')

const dialect = new SqliteDialect({
  database: async () => new Database(dbPath),
})

export const db = new Kysely<DBTypes>({
  dialect,
})
