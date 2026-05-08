import { invoke } from '@tauri-apps/api/core';

export interface Entry {
  id: string;
  created_at: number;
  updated_at: number;
  title: string;
  body: string;
  body_html: string;
  body_text: string;
  mood: number | null;
  pinned: boolean;
  deleted_at: number | null;
  word_count: number;
}

export interface Tag {
  id: string;
  name: string;
}

export interface AppError {
  code: string;
  message: string;
  recoverable: boolean;
}

export interface OkResponse {
  ok: boolean;
}

export interface SearchResult {
  entry: Entry;
  snippet: string;
}

export interface SearchEntriesResponse {
  results: SearchResult[];
  elapsed_ms: number;
}

export function isAppError(error: unknown): error is AppError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error &&
    'recoverable' in error
  );
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
  deleteEntry: (id: string) => invoke<OkResponse>('delete_entry', { id }),
  setEntryPinned: (id: string, pinned: boolean) =>
    invoke<Entry>('set_entry_pinned', { payload: { id, pinned } }),
  createTag: (name: string) => invoke<Tag>('create_tag', { payload: { name } }),
  listTags: () => invoke<Tag[]>('list_tags'),
  getAllTags: () => invoke<Tag[]>('get_all_tags'),
  renameTag: (id: string, name: string) =>
    invoke<Tag>('rename_tag', { payload: { id, name } }),
  deleteTag: (id: string) => invoke<OkResponse>('delete_tag', { id }),
  getTagsForEntry: (entry_id: string) =>
    invoke<Tag[]>('get_tags_for_entry', { entry_id }),
  assignTagToEntry: (entry_id: string, tag_id: string) =>
    invoke<void>('assign_tag_to_entry', { entry_id, tag_id }),
  removeTagFromEntry: (entry_id: string, tag_id: string) =>
    invoke<void>('remove_tag_from_entry', { entry_id, tag_id }),
  getAllEntryTags: () => invoke<Record<string, Tag[]>>('get_all_entry_tags'),
  searchEntries: (query: string, limit: number = 50, offset: number = 0) =>
    invoke<SearchEntriesResponse>('search_entries', {
      payload: { query, limit, offset }
    })
};
