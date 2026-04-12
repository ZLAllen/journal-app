import { invoke } from '@tauri-apps/api/core';

export interface Entry {
  id: string;
  created_at: number;
  updated_at: number;
  body: string;
  mood: number | null;
  pinned: boolean;
  deleted_at: number | null;
}

export interface Tag {
  id: string;
  name: string;
}

export const api = {
  createEntry: (body: string, mood: number | null = null) =>
    invoke<Entry>('create_entry', { payload: { body, mood } }),
  getEntries: () => invoke<Entry[]>('get_entries'),
  updateEntry: (id: string, body: string, mood: number | null = null) =>
    invoke<Entry>('update_entry', { payload: { id, body, mood } }),
  deleteEntry: (id: string) => invoke<void>('delete_entry', { id }),
  createTag: (name: string) => invoke<Tag>('create_tag', { payload: { name } }),
  getAllTags: () => invoke<Tag[]>('get_all_tags')
};
