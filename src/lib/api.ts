import { invoke } from '@tauri-apps/api/core';

export interface Entry {
  id: string;
  created_at: number;
  updated_at: number;
  title: string;
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
  createEntry: (title: string, body: string, mood: number | null = null) =>
    invoke<Entry>('create_entry', { payload: { title, body, mood } }),
  getEntries: () => invoke<Entry[]>('get_entries'),
  updateEntry: (
    id: string,
    title: string,
    body: string,
    mood: number | null = null,
    created_at: number | null = null
  ) => invoke<Entry>('update_entry', { payload: { id, title, body, mood, created_at } }),
  deleteEntry: (id: string) => invoke<void>('delete_entry', { id }),
  createTag: (name: string) => invoke<Tag>('create_tag', { payload: { name } }),
  getAllTags: () => invoke<Tag[]>('get_all_tags'),
  getTagsForEntry: (entry_id: string) =>
    invoke<Tag[]>('get_tags_for_entry', { entryId: entry_id }),
  assignTagToEntry: (entry_id: string, tag_id: string) =>
    invoke<void>('assign_tag_to_entry', { entryId: entry_id, tagId: tag_id }),
  removeTagFromEntry: (entry_id: string, tag_id: string) =>
    invoke<void>('remove_tag_from_entry', { entryId: entry_id, tagId: tag_id })
};
