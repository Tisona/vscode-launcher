<script lang="ts">
  export let message: string;
  export let kind: "warn" | "error" = "warn";
</script>

<div class="banner {kind}" role="alert">{message}</div>

<style>
  .banner {
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.5rem;
    border-radius: 4px;
    font-size: 0.9rem;
  }
  .banner.warn { background: #3a2d00; color: #ffcc66; }
  .banner.error { background: #3a0000; color: #ff9999; }
</style>
