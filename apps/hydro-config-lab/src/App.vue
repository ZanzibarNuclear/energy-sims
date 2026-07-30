<script setup lang="ts">
import ServerStatus from "./components/ServerStatus.vue";
import SiteCanvasPlaceholder from "./components/SiteCanvasPlaceholder.vue";
</script>

<template>
  <div class="app">
    <header class="top">
      <div class="brand">
        <h1>Hydro Config Lab</h1>
        <p class="tagline">
          Prototype plant layouts against the production energy-sim engine
        </p>
      </div>
      <div class="top-actions">
        <span class="config-name" title="Named configs in Lab-PR4">Untitled site</span>
        <button type="button" class="btn" disabled title="Lab-PR4">Save</button>
        <button type="button" class="btn" disabled title="Lab-PR4">Export</button>
        <button type="button" class="btn" disabled title="Lab-PR4">Import…</button>
      </div>
    </header>

    <ServerStatus />

    <div class="workspace">
      <SiteCanvasPlaceholder />

      <aside class="side" aria-label="Properties">
        <h2>Properties</h2>
        <p class="placeholder">
          Select a site element after construction tools land (Lab-PR2). Plant-wide stream,
          penstock, and turbine fields follow in Lab-PR3.
        </p>
        <div class="preview-card">
          <h3>Steady preview</h3>
          <p class="placeholder">Available once the site compiles to a plant and the engine is online.</p>
        </div>
      </aside>
    </div>

    <section class="trial" aria-label="Trial runner">
      <div class="trial-bar">
        <h2>Trial</h2>
        <button type="button" class="btn primary" disabled title="Lab-PR5">Run</button>
        <button type="button" class="btn" disabled title="Lab-PR5">Stop</button>
      </div>
      <p class="placeholder">
        Session trials (ramps, series charts) ship in Lab-PR5 — always against
        <code>energy-sim-server</code>, not browser-only math.
      </p>
    </section>

    <footer class="foot">
      <a href="../../docs/hydro-config-lab.md">Design</a>
      <span class="sep">·</span>
      <a href="../../docs/plans/hydro-config-lab.md">Plan</a>
      <span class="sep">·</span>
      <span>Lab-PR1 scaffold</span>
    </footer>
  </div>
</template>

<style scoped>
.app {
  max-width: 72rem;
  margin: 0 auto;
  padding: 1.25rem 1.25rem 2.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.top {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.brand h1 {
  margin: 0;
  font-size: 1.45rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.tagline {
  margin: 0.25rem 0 0;
  color: var(--muted-fg);
  font-size: 0.9rem;
}

.top-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem;
}

.config-name {
  font-size: 0.85rem;
  color: var(--muted-fg);
  padding: 0.3rem 0.55rem;
  border: 1px dashed var(--border);
  border-radius: 6px;
  margin-right: 0.25rem;
}

.btn {
  font: inherit;
  font-size: 0.85rem;
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--fg);
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.workspace {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(14rem, 18rem);
  gap: 1rem;
  align-items: stretch;
}

@media (max-width: 840px) {
  .workspace {
    grid-template-columns: 1fr;
  }
}

.side {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  padding: 0.9rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.side h2,
.trial h2 {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 650;
}

.placeholder {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.45;
  color: var(--muted-fg);
}

.placeholder code {
  font-size: 0.8em;
  background: var(--code-bg);
  padding: 0.05rem 0.3rem;
  border-radius: 3px;
}

.preview-card {
  margin-top: auto;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.preview-card h3 {
  margin: 0 0 0.4rem;
  font-size: 0.82rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.trial {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  padding: 0.85rem 1rem 1rem;
}

.trial-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.trial-bar h2 {
  margin-right: auto;
}

.foot {
  font-size: 0.78rem;
  color: var(--muted-fg);
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}

.foot a {
  color: var(--accent);
  text-decoration: none;
}

.foot a:hover {
  text-decoration: underline;
}

.sep {
  opacity: 0.5;
}
</style>
