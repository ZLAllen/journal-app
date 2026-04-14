<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Entry, Tag } from '../lib/api';

  type TimelineEntry = Entry & { tags: Tag[] };

  export let entries: TimelineEntry[] = [];
  export let selectedEntryId = '';

  const dispatch = createEventDispatcher<{ select: string }>();

  const moodOptions = [
    { value: 1, label: 'Low', icon: ':(', tone: 'mood-1' },
    { value: 2, label: 'Off', icon: ':/', tone: 'mood-2' },
    { value: 3, label: 'Even', icon: ':|', tone: 'mood-3' },
    { value: 4, label: 'Good', icon: ':)', tone: 'mood-4' },
    { value: 5, label: 'Great', icon: ':D', tone: 'mood-5' }
  ] as const;

  function moodOptionFor(value: number) {
    return moodOptions.find((option) => option.value === value) ?? null;
  }

  function selectEntry(entryId: string): void {
    dispatch('select', entryId);
  }

  function entryTitle(title: string): string {
    const normalized = title.trim();
    return normalized.length > 0 ? normalized : 'Untitled entry';
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }
</script>

<section class="timeline">
  <header>
    <h2>Timeline</h2>
    <p>{entries.length} entries</p>
  </header>

  {#if entries.length === 0}
    <p class="empty">Write your first entry to populate the timeline.</p>
  {:else}
    <ul>
      {#each entries as entry (entry.id)}
        <li>
          <button
            type="button"
            class:selected={entry.id === selectedEntryId}
            on:click={() => selectEntry(entry.id)}
          >
            <div class="row">
              <span class="date">{formatDate(entry.created_at)}</span>
              {#if entry.mood && moodOptionFor(entry.mood)}
                <span
                  class={`mood ${moodOptionFor(entry.mood)?.tone ?? ''}`}
                  title={`Mood ${entry.mood}/5: ${moodOptionFor(entry.mood)?.label ?? ''}`}
                  aria-label={`Mood ${entry.mood}/5`}
                >
                  {moodOptionFor(entry.mood)?.icon}
                </span>
              {/if}
            </div>
            <p class="title">{entryTitle(entry.title)}</p>
            {#if entry.tags.length > 0}
              <div class="tags">
                {#each entry.tags as tag (tag.id)}
                  <span>{tag.name}</span>
                {/each}
              </div>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .timeline {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  h2 {
    margin: 0;
    font-size: 1.2rem;
  }

  header p {
    margin: 0;
    font-size: 0.9rem;
    color: #64748b;
  }

  .empty {
    margin: 0;
    padding: 1rem;
    border: 1px dashed #cbd5e1;
    border-radius: 0.75rem;
    color: #475569;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    max-height: calc(100vh - 15rem);
    overflow: auto;
  }

  button {
    width: 100%;
    border: 1px solid #dbe4ee;
    background: #f8fafc;
    color: inherit;
    border-radius: 0.75rem;
    text-align: left;
    padding: 0.75rem;
    cursor: pointer;
  }

  button.selected {
    border-color: #0ea5e9;
    background: #f0f9ff;
  }

  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
  }

  .date {
    font-size: 0.8rem;
    color: #475569;
  }

  .title {
    margin: 0.45rem 0;
    line-height: 1.35;
    color: #0f172a;
    font-weight: 600;
  }

  .mood {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 2.2rem;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    border: 1px solid transparent;
    font-size: 0.78rem;
    font-weight: 700;
    line-height: 1;
    letter-spacing: 0.02em;
  }

  .mood.mood-1 {
    color: #991b1b;
    background: #fee2e2;
    border-color: #fca5a5;
  }

  .mood.mood-2 {
    color: #9a3412;
    background: #ffedd5;
    border-color: #fdba74;
  }

  .mood.mood-3 {
    color: #1d4ed8;
    background: #dbeafe;
    border-color: #93c5fd;
  }

  .mood.mood-4 {
    color: #166534;
    background: #dcfce7;
    border-color: #86efac;
  }

  .mood.mood-5 {
    color: #854d0e;
    background: #fef3c7;
    border-color: #fcd34d;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .tags span {
    font-size: 0.75rem;
    border-radius: 999px;
    padding: 0.18rem 0.5rem;
    background: #e2e8f0;
    color: #334155;
  }
</style>
