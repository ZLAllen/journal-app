<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import Editor from './routes/Editor.svelte';
  import Search from './routes/Search.svelte';
  import Timeline from './routes/Timeline.svelte';
  import { api, isAppError, type Entry, type Tag } from './lib/api';

  type EntryWithTags = Entry & { tags: Tag[] };

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

  const SEARCH_DELAY_MS = 250;

  $: selectedEntry =
    visibleEntries.find((entry) => entry.id === selectedEntryId) ?? (visibleEntries[0] ?? null);

  $: if (!searching && searchQuery.trim().length === 0) {
    visibleEntries = entries;
    searchElapsedMs = null;
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
      error = `Failed to load entries. ${errorMessage(loadError)}`;
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

  async function createNewEntry(): Promise<void> {
    error = '';

    try {
      const entry = await api.createEntry('', '<p></p>', null);
      const withNoTags: EntryWithTags = { ...entry, tags: [] };
      entries = [withNoTags, ...entries].sort(sortEntries);
      if (searchQuery.trim().length === 0) {
        visibleEntries = entries;
      } else {
        void queueSearch(searchQuery);
      }
      selectedEntryId = entry.id;
    } catch (createError: unknown) {
      error = `Failed to create a new entry. ${errorMessage(createError)}`;
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

  function errorMessage(error: unknown): string {
    if (isAppError(error)) {
      return error.message;
    }

    return error instanceof Error ? error.message : String(error);
  }

  function onEntrySelect(event: CustomEvent<string>): void {
    selectedEntryId = event.detail;
  }

  function onEntrySaved(event: CustomEvent<{ entry: Entry; tags: Tag[] }>): void {
    const { entry, tags } = event.detail;
    entries = entries
      .map((item) => (item.id === entry.id ? { ...entry, tags } : item))
      .sort(sortEntries);

    if (searchQuery.trim().length === 0) {
      visibleEntries = entries;
    } else {
      void queueSearch(searchQuery);
    }
  }

  function onTagsUpdated(event: CustomEvent<{ entryId: string; tags: Tag[] }>): void {
    const { entryId, tags } = event.detail;
    entries = entries.map((item) => (item.id === entryId ? { ...item, tags } : item));
    visibleEntries = visibleEntries.map((item) => (item.id === entryId ? { ...item, tags } : item));
  }

  function onAllTagsUpdated(event: CustomEvent<Tag[]>): void {
    allTags = event.detail;
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
      visibleEntries = entries;
      searchElapsedMs = null;
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
        const byId = new Map(entries.map((entry) => [entry.id, entry]));
        visibleEntries = ids
          .map((id) => byId.get(id))
          .filter((entry): entry is EntryWithTags => Boolean(entry));
        searchElapsedMs = response.elapsed_ms;
      } catch (searchError: unknown) {
        if (revision !== searchRevision) {
          return;
        }

        visibleEntries = [];
        searchElapsedMs = null;
        error = `Search failed. ${errorMessage(searchError)}`;
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
      resultCount={visibleEntries.length}
      elapsedMs={searchElapsedMs}
      noResults={searchQuery.trim().length > 0 && !searching && visibleEntries.length === 0}
      on:queryChange={onSearchQueryChange}
    />

    <section class="layout">
      <aside>
        <Timeline
          entries={visibleEntries}
          {selectedEntryId}
          emptyMessage={
            searchQuery.trim().length > 0
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
