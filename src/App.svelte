<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Editor from './routes/Editor.svelte';
  import Search from './routes/Search.svelte';
  import TagManager from './routes/TagManager.svelte';
  import Timeline from './routes/Timeline.svelte';
  import { api, userMessageForError, type Entry, type Tag } from './lib/api';

  type EntryWithTags = Entry & { tags: Tag[] };
  type SearchFilters = {
    dateFrom: string;
    dateTo: string;
    tagId: string;
    mood: string;
  };

  let entries: EntryWithTags[] = [];
  let visibleEntries: EntryWithTags[] = [];
  let allTags: Tag[] = [];
  let selectedEntryId = '';
  let selectedEntry: EntryWithTags | null = null;
  let loading = true;
  let error = '';
  let searchQuery = '';
  let searchElapsedMs: number | null = null;
  let searching = false;
  let searchTimer: ReturnType<typeof setTimeout> | null = null;
  let searchRevision = 0;
  let searchPanel: Search | null = null;
  let searchSnippets: Record<string, string> = {};
  let tagCounts: Record<string, number> = {};
  let filters: SearchFilters = {
    dateFrom: '',
    dateTo: '',
    tagId: '',
    mood: ''
  };

  const SEARCH_DELAY_MS = 250;

  $: tagCounts = buildTagCounts(entries);

  $: selectedEntry =
    visibleEntries.find((entry) => entry.id === selectedEntryId) ?? (visibleEntries[0] ?? null);

  $: if (!searching && searchQuery.trim().length === 0 && !hasActiveFilters()) {
    visibleEntries = entries;
    searchElapsedMs = null;
    searchSnippets = {};
  }

  onMount(() => {
    window.addEventListener('keydown', handleWindowKeydown);
    void loadInitialData();
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleWindowKeydown);
    if (searchTimer) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
  });

  async function loadInitialData(): Promise<void> {
    loading = true;
    error = '';

    try {
      const [rawEntries, tags] = await Promise.all([api.getEntries(), api.getAllTags()]);
      allTags = tags;
      entries = await withTags(rawEntries);
      visibleEntries = entries;

      if (entries.length > 0) {
        selectedEntryId = entries[0].id;
      }
    } catch (loadError: unknown) {
      error = userMessageForError(loadError, {
        defaultMessage: 'Failed to load entries.',
        databaseMessage: 'Failed to load entries from local database.'
      });
    } finally {
      loading = false;
    }
  }

  async function withTags(rawEntries: Entry[]): Promise<EntryWithTags[]> {
    const entryTagsMap = await api.getAllEntryTags();
    return rawEntries
      .map((entry) => ({ ...entry, tags: entryTagsMap[entry.id] ?? [] }))
      .sort(sortEntries);
  }

  function sortEntries(a: EntryWithTags, b: EntryWithTags): number {
    if (a.pinned !== b.pinned) {
      return a.pinned ? -1 : 1;
    }

    return b.created_at - a.created_at;
  }

  function applyFilters(items: EntryWithTags[]): EntryWithTags[] {
    const fromTime = filters.dateFrom ? new Date(`${filters.dateFrom}T00:00:00`).getTime() : null;
    const toTime = filters.dateTo ? new Date(`${filters.dateTo}T23:59:59.999`).getTime() : null;
    const moodValue = filters.mood ? Number(filters.mood) : null;

    return items.filter((entry) => {
      if (fromTime !== null && entry.created_at < fromTime) {
        return false;
      }

      if (toTime !== null && entry.created_at > toTime) {
        return false;
      }

      if (filters.tagId && !entry.tags.some((tag) => tag.id === filters.tagId)) {
        return false;
      }

      if (moodValue !== null && entry.mood !== moodValue) {
        return false;
      }

      return true;
    });
  }

  function buildTagCounts(items: EntryWithTags[]): Record<string, number> {
    const counts: Record<string, number> = {};
    for (const entry of items) {
      for (const tag of entry.tags) {
        counts[tag.id] = (counts[tag.id] ?? 0) + 1;
      }
    }
    return counts;
  }

  async function createNewEntry(): Promise<void> {
    error = '';

    try {
      const entry = await api.createEntry('', '<p></p>', null);
      const withNoTags: EntryWithTags = { ...entry, tags: [] };
      entries = [withNoTags, ...entries].sort(sortEntries);
      void queueSearch(searchQuery);
      selectedEntryId = entry.id;
    } catch (createError: unknown) {
      error = userMessageForError(createError, {
        defaultMessage: 'Failed to create a new entry.',
        invalidInputMessage: 'Entry data was invalid. Review fields and try again.'
      });
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (!(event.ctrlKey || event.metaKey)) {
      return;
    }

    if (event.key.toLowerCase() === 'n') {
      event.preventDefault();
      void createNewEntry();
      return;
    }

    if (event.key.toLowerCase() === 'f') {
      event.preventDefault();
      searchPanel?.focusSearch();
    }
  }

  function onEntrySelect(event: CustomEvent<string>): void {
    selectedEntryId = event.detail;
  }

  function onEntrySaved(event: CustomEvent<{ entry: Entry; tags: Tag[] }>): void {
    const { entry, tags } = event.detail;
    entries = entries
      .map((item) => (item.id === entry.id ? { ...entry, tags } : item))
      .sort(sortEntries);

    void queueSearch(searchQuery);
  }

  function onTagsUpdated(event: CustomEvent<{ entryId: string; tags: Tag[] }>): void {
    const { entryId, tags } = event.detail;
    entries = entries.map((item) => (item.id === entryId ? { ...item, tags } : item));
    visibleEntries = visibleEntries.map((item) => (item.id === entryId ? { ...item, tags } : item));
    void queueSearch(searchQuery);
  }

  function onAllTagsUpdated(event: CustomEvent<Tag[]>): void {
    allTags = event.detail;
  }

  async function onTagRenamed(event: CustomEvent<{ id: string; name: string }>): Promise<void> {
    const { id, name } = event.detail;
    error = '';
    try {
      const renamed = await api.renameTag(id, name);
      allTags = allTags
        .map((tag) => (tag.id === renamed.id ? renamed : tag))
        .sort((a, b) => a.name.localeCompare(b.name));
      entries = entries.map((entry) => ({
        ...entry,
        tags: entry.tags
          .map((tag) => (tag.id === renamed.id ? renamed : tag))
          .sort((a, b) => a.name.localeCompare(b.name))
      }));
      visibleEntries = visibleEntries.map((entry) => ({
        ...entry,
        tags: entry.tags
          .map((tag) => (tag.id === renamed.id ? renamed : tag))
          .sort((a, b) => a.name.localeCompare(b.name))
      }));
      void queueSearch(searchQuery);
    } catch (renameError: unknown) {
      error = userMessageForError(renameError, {
        defaultMessage: 'Failed to rename tag.',
        invalidInputMessage: 'Tag name is invalid or already exists.',
        notFoundMessage: 'Tag not found.'
      });
    }
  }

  async function onTagDeleted(event: CustomEvent<{ id: string }>): Promise<void> {
    const { id } = event.detail;
    error = '';
    try {
      await api.deleteTag(id);
      allTags = allTags.filter((tag) => tag.id !== id);
      entries = entries.map((entry) => ({
        ...entry,
        tags: entry.tags.filter((tag) => tag.id !== id)
      }));
      visibleEntries = visibleEntries.map((entry) => ({
        ...entry,
        tags: entry.tags.filter((tag) => tag.id !== id)
      }));
      if (filters.tagId === id) {
        filters = { ...filters, tagId: '' };
      }
      void queueSearch(searchQuery);
    } catch (deleteError: unknown) {
      error = userMessageForError(deleteError, {
        defaultMessage: 'Failed to delete tag.',
        notFoundMessage: 'Tag not found.'
      });
    }
  }

  function onEntryDeleted(event: CustomEvent<{ entryId: string }>): void {
    const { entryId } = event.detail;
    const remaining = entries.filter((item) => item.id !== entryId);
    entries = remaining;
    visibleEntries = visibleEntries.filter((item) => item.id !== entryId);

    if (remaining.length === 0) {
      selectedEntryId = '';
      return;
    }

    if (selectedEntryId === entryId) {
      selectedEntryId = remaining[0].id;
    }
  }

  function onSearchQueryChange(event: CustomEvent<string>): void {
    searchQuery = event.detail;
    void queueSearch(searchQuery);
  }

  function onFiltersChange(event: CustomEvent<SearchFilters>): void {
    filters = event.detail;
    void queueSearch(searchQuery);
  }

  function hasActiveFilters(): boolean {
    return Boolean(filters.dateFrom || filters.dateTo || filters.tagId || filters.mood);
  }

  async function queueSearch(query: string): Promise<void> {
    const trimmed = query.trim();
    searchRevision += 1;
    const revision = searchRevision;

    if (searchTimer) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }

    if (trimmed.length === 0) {
      searching = false;
      visibleEntries = applyFilters(entries);
      searchElapsedMs = null;
      searchSnippets = {};
      return;
    }

    searching = true;
    searchTimer = setTimeout(async () => {
      try {
        const response = await api.searchEntries(trimmed, 200, 0);
        if (revision !== searchRevision) {
          return;
        }

        const ids = response.results.map((result) => result.entry.id);
        searchSnippets = Object.fromEntries(
          response.results.map((result) => [result.entry.id, result.snippet])
        );
        const byId = new Map(entries.map((entry) => [entry.id, entry]));
        const matchedEntries = ids
          .map((id) => byId.get(id))
          .filter((entry): entry is EntryWithTags => Boolean(entry));
        visibleEntries = applyFilters(matchedEntries);
        searchElapsedMs = response.elapsed_ms;
      } catch (searchError: unknown) {
        if (revision !== searchRevision) {
          return;
        }

        visibleEntries = [];
        searchElapsedMs = null;
        searchSnippets = {};
        error = userMessageForError(searchError, {
          defaultMessage: 'Search failed.',
          invalidInputMessage: 'Search query is invalid.'
        });
      } finally {
        if (revision === searchRevision) {
          searching = false;
        }
      }
    }, SEARCH_DELAY_MS);
  }
</script>

<main>
  <header>
    <div>
      <h1>Journal</h1>
      <p>Write freely. Everything auto-saves in 5 seconds.</p>
    </div>
    <button type="button" aria-label="Create new entry" on:click={createNewEntry}>New Entry</button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if loading}
    <p class="loading">Loading journal...</p>
  {:else}
    <Search
      bind:this={searchPanel}
      query={searchQuery}
      {searching}
      {allTags}
      {filters}
      resultCount={visibleEntries.length}
      elapsedMs={searchElapsedMs}
      noResults={!searching && visibleEntries.length === 0 && (searchQuery.trim().length > 0 || hasActiveFilters())}
      on:queryChange={onSearchQueryChange}
      on:filtersChange={onFiltersChange}
    />

    <TagManager
      tags={allTags}
      counts={tagCounts}
      on:rename={onTagRenamed}
      on:delete={onTagDeleted}
    />

    <section class="layout">
      <aside>
        <Timeline
          entries={visibleEntries}
          {selectedEntryId}
          snippets={searchSnippets}
          query={searchQuery}
          emptyMessage={
            searchQuery.trim().length > 0 || hasActiveFilters()
              ? 'No entries matched your search.'
              : 'Write your first entry to populate the timeline.'
          }
          on:select={onEntrySelect}
          on:create={createNewEntry}
        />
      </aside>
      <article>
        <Editor
          entry={selectedEntry}
          {allTags}
          on:entrySaved={onEntrySaved}
          on:entryDeleted={onEntryDeleted}
          on:tagsUpdated={onTagsUpdated}
          on:allTagsUpdated={onAllTagsUpdated}
        />
      </article>
    </section>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    background: linear-gradient(180deg, #f7f8fa 0%, #eef6ff 100%);
    color: #0f172a;
    font-family: 'IBM Plex Sans', 'Noto Sans', sans-serif;
  }

  main {
    max-width: 1180px;
    margin: 0 auto;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 1rem;
  }

  h1 {
    margin: 0;
    font-size: 1.8rem;
    letter-spacing: -0.02em;
  }

  header p {
    margin: 0.2rem 0 0;
    color: #475569;
  }

  header button {
    border: 0;
    background: #0ea5e9;
    color: #fff;
    border-radius: 0.65rem;
    padding: 0.55rem 0.9rem;
    cursor: pointer;
    font-weight: 600;
    width: auto;
    align-self: flex-end;
    flex: 0 0 auto;
  }

  button:focus-visible {
    outline: 2px solid #0284c7;
    outline-offset: 2px;
  }

  .layout {
    display: grid;
    grid-template-columns: 320px minmax(0, 1fr);
    gap: 1rem;
  }

  aside,
  article {
    border: 1px solid #dbe4ee;
    border-radius: 1rem;
    padding: 0.9rem;
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(4px);
  }

  .error {
    margin: 0;
    color: #b91c1c;
  }

  .loading {
    margin: 0;
    color: #334155;
  }

  @media (max-width: 900px) {
    .layout {
      grid-template-columns: 1fr;
    }

    aside {
      order: 2;
    }
  }
</style>
