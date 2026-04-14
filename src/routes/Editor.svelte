<script lang="ts">
  import { createEventDispatcher, onDestroy, onMount } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import type { Entry, Tag } from '../lib/api';
  import { api } from '../lib/api';

  type EntryWithTags = Entry & { tags: Tag[] };
  type SaveState = 'idle' | 'pending' | 'saving' | 'saved' | 'error';

  export let entry: EntryWithTags | null = null;
  export let allTags: Tag[] = [];

  const dispatch = createEventDispatcher<{
    entrySaved: { entry: Entry; tags: Tag[] };
    tagsUpdated: { entryId: string; tags: Tag[] };
    allTagsUpdated: Tag[];
  }>();

  const moodOptions = [
    { value: 1, label: 'Low', icon: ':(', tone: 'mood-1' },
    { value: 2, label: 'Off', icon: ':/', tone: 'mood-2' },
    { value: 3, label: 'Even', icon: ':|', tone: 'mood-3' },
    { value: 4, label: 'Good', icon: ':)', tone: 'mood-4' },
    { value: 5, label: 'Great', icon: ':D', tone: 'mood-5' }
  ] as const;

  let tiptapEditor: Editor | null = null;
  let title = '';
  let bodyHtml = '';
  let mood: number | null = null;
  let entryTags: Tag[] = [];
  let dateInput = '';
  let createdAtMs: number | null = null;
  let baselineCreatedAtMs: number | null = null;
  let currentEntryId = '';
  let saveState: SaveState = 'idle';
  let saveError = '';
  let tagDraft = '';
  let dirty = false;
  let syncingFromEntry = false;
  let switchingEntry = false;
  let pendingEntry: EntryWithTags | null = null;
  let saveInFlight = false;
  let queuedSave = false;
  let changeRevision = 0;
  let lastUserEditAt = 0;
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let autosaveTickTimer: ReturnType<typeof setInterval> | null = null;
  let autosaveDueAt = 0;
  let autosaveNowMs = 0;
  let autosaveRemainingMs = 0;
  let autosaveProgress = 0;
  let autosaveLabel = '0.0s';
  let saveStateResetTimer: ReturnType<typeof setTimeout> | null = null;
  let editorRoot: HTMLDivElement | null = null;

  const AUTOSAVE_DELAY_MS = 5000;
  const ACTIVE_EDIT_GRACE_MS = 1200;

  $: autosaveRemainingMs =
    saveState === 'pending' ? Math.max(0, autosaveDueAt - autosaveNowMs) : 0;
  $: autosaveProgress =
    saveState === 'pending'
      ? Math.min(100, Math.max(0, ((AUTOSAVE_DELAY_MS - autosaveRemainingMs) / AUTOSAVE_DELAY_MS) * 100))
      : 0;
  $: autosaveLabel = `${Math.max(0.1, autosaveRemainingMs / 1000).toFixed(1)}s`;

  onMount(() => {
    if (!editorRoot) {
      return;
    }

    tiptapEditor = new Editor({
      element: editorRoot,
      extensions: [StarterKit],
      content: '<p></p>',
      onUpdate: ({ editor: editorInstance }) => {
        bodyHtml = editorInstance.getHTML();
        if (!syncingFromEntry) {
          scheduleSave();
        }
      }
    });

    if (entry) {
      syncFromEntry(entry);
    }
  });

  onDestroy(() => {
    clearPendingTimers();
    tiptapEditor?.destroy();
  });

  $: if (entry && tiptapEditor) {
    if (entry.id !== currentEntryId) {
      if (dirty && currentEntryId) {
        pendingEntry = entry;
        if (!switchingEntry) {
          void flushBeforeSwitch();
        }
      } else {
        syncFromEntry(entry);
      }
    } else if (!dirty) {
      syncFromEntry(entry);
    }
  }

  $: if (!entry) {
    currentEntryId = '';
  }

  function syncFromEntry(nextEntry: EntryWithTags): void {
    clearPendingSaveTimer();
    stopAutosaveProgress();
    syncingFromEntry = true;
    currentEntryId = nextEntry.id;
    title = nextEntry.title;
    bodyHtml = nextEntry.body;
    mood = nextEntry.mood;
    entryTags = [...nextEntry.tags];
    createdAtMs = nextEntry.created_at;
    baselineCreatedAtMs = nextEntry.created_at;
    dateInput = toInputDate(nextEntry.created_at);
    tiptapEditor?.commands.setContent(nextEntry.body || '<p></p>', { emitUpdate: false });
    dirty = false;
    saveState = 'idle';
    saveError = '';
    queuedSave = false;
    changeRevision = 0;
    syncingFromEntry = false;
  }

  async function flushBeforeSwitch(): Promise<void> {
    if (!pendingEntry) {
      return;
    }

    switchingEntry = true;
    clearPendingSaveTimer();

    if (dirty) {
      await persistEntry();
      if (dirty || saveState === 'error') {
        switchingEntry = false;
        return;
      }
    }

    const nextEntry = pendingEntry;
    pendingEntry = null;
    if (nextEntry) {
      syncFromEntry(nextEntry);
    }

    switchingEntry = false;
  }

  function clearPendingTimers(): void {
    clearPendingSaveTimer();
    stopAutosaveProgress();
    if (saveStateResetTimer) {
      clearTimeout(saveStateResetTimer);
      saveStateResetTimer = null;
    }
  }

  function clearPendingSaveTimer(): void {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
    }
  }

  function startAutosaveProgress(delayMs: number): void {
    autosaveDueAt = Date.now() + delayMs;
    autosaveNowMs = Date.now();
    if (autosaveTickTimer) {
      clearInterval(autosaveTickTimer);
    }
    autosaveTickTimer = setInterval(() => {
      autosaveNowMs = Date.now();
    }, 100);
  }

  function stopAutosaveProgress(): void {
    if (autosaveTickTimer) {
      clearInterval(autosaveTickTimer);
      autosaveTickTimer = null;
    }
    autosaveDueAt = 0;
    autosaveNowMs = 0;
  }

  function queueSaveAttempt(delayMs: number): void {
    clearPendingSaveTimer();
    startAutosaveProgress(delayMs);
    saveTimer = setTimeout(() => {
      const idleForMs = Date.now() - lastUserEditAt;
      if (idleForMs < ACTIVE_EDIT_GRACE_MS) {
        queueSaveAttempt(ACTIVE_EDIT_GRACE_MS - idleForMs);
        return;
      }

      void persistEntry();
    }, delayMs);
  }

  function toInputDate(timestamp: number): string {
    const date = new Date(timestamp);
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${date.getFullYear()}-${month}-${day}`;
  }

  function fromInputDate(dateValue: string, fallbackTimestamp: number): number {
    const [year, month, day] = dateValue.split('-').map((part) => Number(part));
    const source = new Date(fallbackTimestamp);
    source.setFullYear(year, month - 1, day);
    return source.getTime();
  }

  function scheduleSave(): void {
    if (!currentEntryId) {
      return;
    }

    lastUserEditAt = Date.now();
    changeRevision += 1;
    dirty = true;
    saveState = 'pending';
    saveError = '';
    queueSaveAttempt(AUTOSAVE_DELAY_MS);
  }

  async function persistEntry(): Promise<void> {
    if (!currentEntryId || !dirty) {
      return;
    }

    if (saveInFlight) {
      queuedSave = true;
      return;
    }

    const saveRevision = changeRevision;
    const targetId = currentEntryId;
    const titleSnapshot = title.trim();
    const bodySnapshot = bodyHtml;
    const moodSnapshot = mood;
    const backdatedCreatedAt =
      createdAtMs !== null && baselineCreatedAtMs !== null && createdAtMs !== baselineCreatedAtMs
        ? createdAtMs
        : null;

    saveInFlight = true;
    queuedSave = false;

    try {
      stopAutosaveProgress();
      saveState = 'saving';
      const updatedEntry = await api.updateEntry(
        targetId,
        titleSnapshot,
        bodySnapshot,
        moodSnapshot,
        backdatedCreatedAt
      );
      dispatch('entrySaved', { entry: updatedEntry, tags: entryTags });
      baselineCreatedAtMs = updatedEntry.created_at;
      if (changeRevision === saveRevision) {
        dirty = false;
        saveState = 'saved';
      } else {
        saveState = 'idle';
      }
      saveError = '';

      if (saveStateResetTimer) {
        clearTimeout(saveStateResetTimer);
      }

      if (!dirty) {
        saveStateResetTimer = setTimeout(() => {
          if (!dirty && saveState === 'saved') {
            saveState = 'idle';
          }
        }, 1200);
      }
    } catch (error: unknown) {
      saveState = 'error';
      saveError = error instanceof Error ? error.message : String(error);
    } finally {
      stopAutosaveProgress();
      saveInFlight = false;
      if (queuedSave) {
        queueSaveAttempt(300);
      }
    }
  }

  function updateTitle(event: Event): void {
    const target = event.target as HTMLInputElement;
    title = target.value;
    scheduleSave();
  }

  function setMood(nextMood: number): void {
    mood = mood === nextMood ? null : nextMood;
    scheduleSave();
  }

  function updateDate(event: Event): void {
    if (!entry) {
      return;
    }

    const target = event.target as HTMLInputElement;
    dateInput = target.value;
    createdAtMs = fromInputDate(target.value, entry.created_at);
    scheduleSave();
  }

  async function addTag(): Promise<void> {
    if (!entry) {
      return;
    }

    const normalized = tagDraft.trim();
    if (!normalized) {
      return;
    }

    const existing = allTags.find((tag) => tag.name.toLowerCase() === normalized.toLowerCase());
    const tag = existing ?? (await api.createTag(normalized));

    if (!existing) {
      dispatch('allTagsUpdated', [...allTags, tag].sort((a, b) => a.name.localeCompare(b.name)));
    }

    if (!entryTags.some((item) => item.id === tag.id)) {
      await api.assignTagToEntry(entry.id, tag.id);
      entryTags = [...entryTags, tag].sort((a, b) => a.name.localeCompare(b.name));
      dispatch('tagsUpdated', { entryId: entry.id, tags: entryTags });
    }

    tagDraft = '';
  }

  async function removeTag(tagId: string): Promise<void> {
    if (!entry) {
      return;
    }

    await api.removeTagFromEntry(entry.id, tagId);
    entryTags = entryTags.filter((tag) => tag.id !== tagId);
    dispatch('tagsUpdated', { entryId: entry.id, tags: entryTags });
  }

  function onTagKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      void addTag();
    }
  }
</script>

<section class="editor-wrap">
  {#if !entry}
    <p class="empty">Select an entry from the timeline to edit it.</p>
  {:else}
    <header>
      <h2>Editor</h2>
      <div class="status" data-state={saveState} aria-live="polite">
        {#if saveState === 'pending'}Saving in {autosaveLabel}{/if}
        {#if saveState === 'saving'}<span class="saving-dot" aria-hidden="true"></span>Saving...{/if}
        {#if saveState === 'saved'}Saved{/if}
        {#if saveState === 'error'}Save failed{/if}
      </div>
    </header>

    {#if saveState === 'pending'}
      <div class="save-progress" aria-hidden="true">
        <span style={`width: ${autosaveProgress}%`}></span>
      </div>
    {/if}

    {#if saveState === 'error' && saveError}
      <p class="save-error" title={saveError}>{saveError}</p>
    {/if}

    <label class="title-field">
      Title
      <input type="text" value={title} placeholder="Add a title" on:input={updateTitle} />
    </label>

    <div class="toolbar">
      <button
        type="button"
        class:active={tiptapEditor?.isActive('bold')}
        on:click={() => tiptapEditor?.chain().focus().toggleBold().run()}
      >
        Bold
      </button>
      <button
        type="button"
        class:active={tiptapEditor?.isActive('italic')}
        on:click={() => tiptapEditor?.chain().focus().toggleItalic().run()}
      >
        Italic
      </button>
      <button
        type="button"
        class:active={tiptapEditor?.isActive('heading', { level: 2 })}
        on:click={() => tiptapEditor?.chain().focus().toggleHeading({ level: 2 }).run()}
      >
        Heading
      </button>
      <button
        type="button"
        class:active={tiptapEditor?.isActive('bulletList')}
        on:click={() => tiptapEditor?.chain().focus().toggleBulletList().run()}
      >
        Bullet List
      </button>
    </div>

    <div class="meta">
      <label>
        Date
        <input type="date" value={dateInput} on:change={updateDate} />
      </label>

      <div class="moods">
        <span>Mood</span>
        <div>
          {#each moodOptions as moodOption}
            <button
              type="button"
              class:selected={mood === moodOption.value}
              class={moodOption.tone}
              title={`Mood ${moodOption.value}/5: ${moodOption.label}`}
              aria-label={`Set mood to ${moodOption.label}`}
              on:click={() => setMood(moodOption.value)}
            >
              <span class="mood-icon" aria-hidden="true">{moodOption.icon}</span>
            </button>
          {/each}
        </div>
      </div>
    </div>

    <div class="tags">
      <div class="tag-list">
        {#each entryTags as tag (tag.id)}
          <span>
            {tag.name}
            <button type="button" aria-label={`Remove ${tag.name}`} on:click={() => removeTag(tag.id)}>
              x
            </button>
          </span>
        {/each}
      </div>

      <div class="tag-input">
        <input
          type="text"
          bind:value={tagDraft}
          list="tag-suggestions"
          placeholder="Add a tag"
          on:keydown={onTagKeydown}
        />
        <button type="button" on:click={addTag}>Add Tag</button>
      </div>

      <datalist id="tag-suggestions">
        {#each allTags as tag (tag.id)}
          <option value={tag.name}></option>
        {/each}
      </datalist>
    </div>

    <div class="editor" bind:this={editorRoot}></div>
  {/if}
</section>

<style>
  .editor-wrap {
    min-height: 70vh;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .empty {
    margin: 0;
    color: #475569;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  .status {
    min-width: 4.5rem;
    text-align: right;
    font-size: 0.85rem;
    color: #64748b;
  }

  .status[data-state='error'] {
    color: #dc2626;
  }

  .status[data-state='pending'] {
    color: #0369a1;
  }

  .save-progress {
    height: 0.28rem;
    width: 100%;
    border-radius: 999px;
    background: #e2e8f0;
    overflow: hidden;
  }

  .save-progress span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, #0ea5e9, #22d3ee);
    transition: width 90ms linear;
  }

  .saving-dot {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 999px;
    display: inline-block;
    margin-right: 0.35rem;
    background: #0ea5e9;
    animation: savingPulse 1s ease-in-out infinite;
  }

  @keyframes savingPulse {
    0% {
      opacity: 0.35;
      transform: scale(0.9);
    }

    50% {
      opacity: 1;
      transform: scale(1.1);
    }

    100% {
      opacity: 0.35;
      transform: scale(0.9);
    }
  }

  .save-error {
    margin: -0.2rem 0 0;
    color: #b91c1c;
    font-size: 0.85rem;
    line-height: 1.3;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .title-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #334155;
  }

  .title-field input {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.48rem 0.6rem;
    font-size: 0.95rem;
  }

  .toolbar button,
  .moods button,
  .tag-input button {
    border: 1px solid #cbd5e1;
    background: #ffffff;
    border-radius: 0.5rem;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
    font-size: 0.9rem;
    width: auto;
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .toolbar button.active {
    border-color: #0ea5e9;
    background: #f0f9ff;
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: center;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #334155;
  }

  input[type='date'] {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.4rem 0.55rem;
  }

  .moods {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .moods > div {
    display: flex;
    gap: 0.3rem;
  }

  .moods button {
    min-width: 2.5rem;
    padding: 0.3rem 0.5rem;
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .moods button.selected {
    border-color: #0ea5e9;
    background: #f0f9ff;
  }

  .mood-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 1.6rem;
    line-height: 1;
  }

  .moods button.mood-1 {
    color: #991b1b;
    background: #fee2e2;
    border-color: #fca5a5;
  }

  .moods button.mood-2 {
    color: #9a3412;
    background: #ffedd5;
    border-color: #fdba74;
  }

  .moods button.mood-3 {
    color: #1d4ed8;
    background: #dbeafe;
    border-color: #93c5fd;
  }

  .moods button.mood-4 {
    color: #166534;
    background: #dcfce7;
    border-color: #86efac;
  }

  .moods button.mood-5 {
    color: #854d0e;
    background: #fef3c7;
    border-color: #fcd34d;
  }

  .moods button.selected.mood-1,
  .moods button.selected.mood-2,
  .moods button.selected.mood-3,
  .moods button.selected.mood-4,
  .moods button.selected.mood-5 {
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.7), 0 0 0 3px rgba(14, 165, 233, 0.15);
  }

  .tags {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .tag-list span {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: #e2e8f0;
    border-radius: 999px;
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
  }

  .tag-list span button {
    border: 0;
    background: transparent;
    cursor: pointer;
    color: #334155;
    font-size: 0.8rem;
    padding: 0;
  }

  .tag-input {
    display: flex;
    gap: 0.45rem;
    align-items: center;
  }

  .tag-input input {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.4rem 0.55rem;
    min-width: 12rem;
  }

  .editor {
    border: 1px solid #dbe4ee;
    border-radius: 0.75rem;
    padding: 0.8rem;
    min-height: 22rem;
    background: #fff;
  }

  .editor :global(.ProseMirror) {
    min-height: 20rem;
    outline: none;
    line-height: 1.5;
    color: #0f172a;
  }

  .editor :global(.ProseMirror h2) {
    margin: 0.8rem 0 0.45rem;
    font-size: 1.25rem;
  }

  .editor :global(.ProseMirror ul) {
    padding-left: 1.2rem;
    list-style-type: disc;
  }

  .editor :global(.ProseMirror ol) {
    padding-left: 1.2rem;
    list-style-type: decimal;
  }

  .editor :global(.ProseMirror li) {
    margin: 0.15rem 0;
  }
</style>
