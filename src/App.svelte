<script lang="ts">
  import { onMount } from 'svelte';
  import Editor from './routes/Editor.svelte';
  import Timeline from './routes/Timeline.svelte';
  import { api, type Entry, type Tag } from './lib/api';

  type EntryWithTags = Entry & { tags: Tag[] };

  let entries: EntryWithTags[] = [];
  let allTags: Tag[] = [];
  let selectedEntryId = '';
  let selectedEntry: EntryWithTags | null = null;
  let loading = true;
  let error = '';

  $: selectedEntry =
    entries.find((entry) => entry.id === selectedEntryId) ?? (entries[0] ?? null);

  onMount(() => {
    void loadInitialData();
  });

  async function loadInitialData(): Promise<void> {
    loading = true;
    error = '';

    try {
      const [rawEntries, tags] = await Promise.all([api.getEntries(), api.getAllTags()]);
      allTags = tags;
      entries = await withTags(rawEntries);

      if (entries.length > 0) {
        selectedEntryId = entries[0].id;
      }
    } catch {
      error = 'Failed to load entries. Please restart the app.';
    } finally {
      loading = false;
    }
  }

  async function withTags(rawEntries: Entry[]): Promise<EntryWithTags[]> {
    const entryTagsMap = await api.getAllEntryTags();
    return rawEntries
      .map((entry) => ({ ...entry, tags: entryTagsMap[entry.id] ?? [] }))
      .sort((a, b) => b.created_at - a.created_at);
  }

  async function createNewEntry(): Promise<void> {
    error = '';

    try {
      const entry = await api.createEntry('', '<p></p>', null);
      const withNoTags: EntryWithTags = { ...entry, tags: [] };
      entries = [withNoTags, ...entries].sort((a, b) => b.created_at - a.created_at);
      selectedEntryId = entry.id;
    } catch {
      error = 'Failed to create a new entry.';
    }
  }

  function onEntrySelect(event: CustomEvent<string>): void {
    selectedEntryId = event.detail;
  }

  function onEntrySaved(event: CustomEvent<{ entry: Entry; tags: Tag[] }>): void {
    const { entry, tags } = event.detail;
    entries = entries
      .map((item) => (item.id === entry.id ? { ...entry, tags } : item))
      .sort((a, b) => b.created_at - a.created_at);
  }

  function onTagsUpdated(event: CustomEvent<{ entryId: string; tags: Tag[] }>): void {
    const { entryId, tags } = event.detail;
    entries = entries.map((item) => (item.id === entryId ? { ...item, tags } : item));
  }

  function onAllTagsUpdated(event: CustomEvent<Tag[]>): void {
    allTags = event.detail;
  }
</script>

<main>
  <header>
    <div>
      <h1>Journal</h1>
      <p>Write freely. Everything auto-saves in 5 seconds.</p>
    </div>
    <button type="button" on:click={createNewEntry}>New Entry</button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if loading}
    <p class="loading">Loading journal...</p>
  {:else}
    <section class="layout">
      <aside>
        <Timeline entries={entries} {selectedEntryId} on:select={onEntrySelect} />
      </aside>
      <article>
        <Editor
          entry={selectedEntry}
          {allTags}
          on:entrySaved={onEntrySaved}
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
