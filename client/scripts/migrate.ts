import * as path from 'path';
import { fileURLToPath } from 'url';
import { promises as fs } from 'fs';
import { Kysely, Migrator, SqliteDialect, FileMigrationProvider } from 'kysely';
import { type Database as DBTypes } from '$lib/server/db/db-types';
import Database from "better-sqlite3";

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

async function migrateToLatest() {
  const db = new Kysely<DBTypes>({
    dialect: new SqliteDialect({
      database: new Database(path.resolve(process.cwd(), './db.sqlite')),
    }),
  })

  const migrator = new Migrator({
    db,
    provider: new FileMigrationProvider({
      fs,
      path,
      migrationFolder: path.join(process.cwd(), 'src', 'lib', 'server', 'db', 'migrations'),
    }),
  })

  const { error, results } = await migrator.migrateToLatest()

  for (const result of results ?? []) {
    if (result.status === 'Success') {
      console.log(`Migration "${result.migrationName}" executed successfully.`)
    } else {
      console.error(`Migration "${result.migrationName}" failed.`)
    }
  }

  if (error) {
    console.error('Migration process failed:')
    console.error(error)
    process.exit(1)
  }

  await db.destroy()
}

migrateToLatest()
