<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Tag } from '../lib/api';

  export let tags: Tag[] = [];
  export let counts: Record<string, number> = {};

  const dispatch = createEventDispatcher<{
    rename: { id: string; name: string };
    delete: { id: string };
  }>();

  let renamingId = '';
  let renameDraft = '';

  function startRename(tag: Tag): void {
    renamingId = tag.id;
    renameDraft = tag.name;
  }

  function cancelRename(): void {
    renamingId = '';
    renameDraft = '';
  }

  function submitRename(tagId: string): void {
    const nextName = renameDraft.trim();
    if (!nextName) {
      return;
    }

    dispatch('rename', { id: tagId, name: nextName });
    cancelRename();
  }

  function confirmDelete(tagId: string, tagName: string): void {
    const confirmed = window.confirm(`Delete tag "${tagName}"?`);
    if (!confirmed) {
      return;
    }

    dispatch('delete', { id: tagId });
  }

  function onRenameKeydown(event: KeyboardEvent, tagId: string): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      submitRename(tagId);
      return;
    }

    if (event.key === 'Escape') {
      event.preventDefault();
      cancelRename();
    }
  }
</script>

<section class="tags-admin" aria-label="Tag management">
  <header>
    <h2>Tags</h2>
    <p>{tags.length}</p>
  </header>

  {#if tags.length === 0}
    <p class="empty">No tags yet.</p>
  {:else}
    <ul>
      {#each tags as tag (tag.id)}
        <li>
          <div class="row">
            {#if renamingId === tag.id}
              <input
                type="text"
                bind:value={renameDraft}
                aria-label={`Rename ${tag.name}`}
                on:keydown={(event) => onRenameKeydown(event, tag.id)}
              />
            {:else}
              <span class="name">{tag.name}</span>
            {/if}
            <span class="count">{counts[tag.id] ?? 0}</span>
          </div>
          <div class="actions">
            {#if renamingId === tag.id}
              <button type="button" aria-label={`Save rename for ${tag.name}`} on:click={() => submitRename(tag.id)}>
                Save
              </button>
              <button type="button" aria-label={`Cancel rename for ${tag.name}`} on:click={cancelRename}>
                Cancel
              </button>
            {:else}
              <button type="button" aria-label={`Rename tag ${tag.name}`} on:click={() => startRename(tag)}>
                Rename
              </button>
              <button
                type="button"
                class="danger"
                aria-label={`Delete tag ${tag.name}`}
                on:click={() => confirmDelete(tag.id, tag.name)}
              >
                Delete
              </button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .tags-admin {
    border: 1px solid #dbe4ee;
    border-radius: 0.75rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.85);
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  h2,
  p {
    margin: 0;
  }

  header p {
    color: #64748b;
    font-size: 0.84rem;
  }

  .empty {
    color: #64748b;
    font-size: 0.85rem;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  li {
    border: 1px solid #e2e8f0;
    border-radius: 0.6rem;
    padding: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .name {
    font-size: 0.9rem;
    color: #0f172a;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 160px;
  }

  .count {
    font-size: 0.78rem;
    color: #334155;
    background: #e2e8f0;
    border-radius: 999px;
    padding: 0.1rem 0.45rem;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  button {
    border: 1px solid #cbd5e1;
    background: #fff;
    border-radius: 0.45rem;
    padding: 0.25rem 0.55rem;
    font-size: 0.8rem;
    cursor: pointer;
  }

  button.danger {
    color: #b91c1c;
    border-color: #fca5a5;
    background: #fef2f2;
  }

  input {
    border: 1px solid #cbd5e1;
    border-radius: 0.45rem;
    padding: 0.25rem 0.45rem;
    font-size: 0.85rem;
    width: 10rem;
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid #0284c7;
    outline-offset: 2px;
  }
</style>
