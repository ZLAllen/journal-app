<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Tag } from '../lib/api';

  type SearchFilters = {
    dateFrom: string;
    dateTo: string;
    tagId: string;
    mood: string;
  };

  export let query = '';
  export let searching = false;
  export let resultCount = 0;
  export let elapsedMs: number | null = null;
  export let noResults = false;
  export let allTags: Tag[] = [];
  export let filters: SearchFilters = {
    dateFrom: '',
    dateTo: '',
    tagId: '',
    mood: ''
  };

  const dispatch = createEventDispatcher<{
    queryChange: string;
    filtersChange: SearchFilters;
  }>();

  let inputEl: HTMLInputElement | null = null;

  export function focusSearch(): void {
    inputEl?.focus();
    inputEl?.select();
  }

  function onInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    dispatch('queryChange', target.value);
  }

  function onFilterChange(event: Event): void {
    const target = event.target as HTMLInputElement | HTMLSelectElement;
    dispatch('filtersChange', {
      ...filters,
      [target.name]: target.value
    });
  }
</script>

<section class="search">
  <label>
    Search
    <input
      bind:this={inputEl}
      type="search"
      value={query}
      placeholder="Search entries"
      aria-label="Search entries"
      on:input={onInput}
    />
  </label>

  <div class="filters" aria-label="Search filters">
    <label>
      From
      <input
        type="date"
        name="dateFrom"
        value={filters.dateFrom}
        aria-label="Filter from date"
        on:change={onFilterChange}
      />
    </label>
    <label>
      To
      <input
        type="date"
        name="dateTo"
        value={filters.dateTo}
        aria-label="Filter to date"
        on:change={onFilterChange}
      />
    </label>
    <label>
      Tag
      <select name="tagId" value={filters.tagId} aria-label="Filter by tag" on:change={onFilterChange}>
        <option value="">All tags</option>
        {#each allTags as tag (tag.id)}
          <option value={tag.id}>{tag.name}</option>
        {/each}
      </select>
    </label>
    <label>
      Mood
      <select name="mood" value={filters.mood} aria-label="Filter by mood" on:change={onFilterChange}>
        <option value="">Any mood</option>
        <option value="1">1 - Low</option>
        <option value="2">2 - Off</option>
        <option value="3">3 - Even</option>
        <option value="4">4 - Good</option>
        <option value="5">5 - Great</option>
      </select>
    </label>
  </div>

  <p aria-live="polite">
    {#if searching}
      Searching...
    {:else if query.trim().length > 0}
      {resultCount} result{resultCount === 1 ? '' : 's'}{#if elapsedMs !== null} in {elapsedMs}ms{/if}
    {:else}
      Search by keyword
    {/if}
  </p>

  {#if noResults}
    <p class="no-results">No entries matched your search.</p>
  {/if}
</section>

<style>
  .search {
    border: 1px solid #dbe4ee;
    border-radius: 0.75rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.85);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: #334155;
  }

  input {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.45rem 0.55rem;
    font-size: 0.95rem;
  }

  select {
    border: 1px solid #cbd5e1;
    border-radius: 0.5rem;
    padding: 0.45rem 0.55rem;
    font-size: 0.95rem;
    background: #fff;
  }

  p {
    margin: 0;
    color: #475569;
    font-size: 0.82rem;
  }

  .no-results {
    color: #b45309;
  }

  .filters {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.5rem;
  }

  input:focus-visible {
    outline: 2px solid #0284c7;
    outline-offset: 2px;
  }

  select:focus-visible {
    outline: 2px solid #0284c7;
    outline-offset: 2px;
  }

  @media (max-width: 900px) {
    .filters {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
