<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let query = '';
  export let searching = false;
  export let resultCount = 0;
  export let elapsedMs: number | null = null;
  export let noResults = false;

  const dispatch = createEventDispatcher<{ queryChange: string }>();

  let inputEl: HTMLInputElement | null = null;

  export function focusSearch(): void {
    inputEl?.focus();
    inputEl?.select();
  }

  function onInput(event: Event): void {
    const target = event.target as HTMLInputElement;
    dispatch('queryChange', target.value);
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

  p {
    margin: 0;
    color: #475569;
    font-size: 0.82rem;
  }

  .no-results {
    color: #b45309;
  }

  input:focus-visible {
    outline: 2px solid #0284c7;
    outline-offset: 2px;
  }
</style>
