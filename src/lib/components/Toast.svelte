<script lang="ts">
  import { toasts } from "../toasts";
</script>

<div class="stack">
  {#each $toasts as t (t.id)}
    <div class="toast {t.kind}">{t.message}</div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 300;
    pointer-events: none;
  }
  .toast {
    background: #252526;
    border: 1px solid #454545;
    color: #d4d4d4;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    box-shadow: 0 4px 8px rgba(0,0,0,0.4);
    max-width: 24rem;
    pointer-events: auto;
  }
  .toast.error { border-color: #be1100; }
</style>
