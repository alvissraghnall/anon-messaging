import type { Insertable, Selectable, Updateable } from "kysely"

export interface UsersTable {
  id: string
  username: string
  password: string
}

export interface AnonMappingsTable {
  anon_id: string
  user_id: string
  created_at: number
  expires_at: number
  thread_id: number
}

export interface Database {
  users: UsersTable
  anon_mappings: AnonMappingsTable
}

export type Users = Selectable<UsersTable>
export type NewUser = Insertable<UsersTable>
export type UsersUpdate = Updateable<UsersTable>

export type AnonMappings = Selectable<AnonMappingsTable>
export type NewAnonMapping = Insertable<AnonMappingsTable>
export type AnonMappingsUpdate = Updateable<AnonMappingsTable>
