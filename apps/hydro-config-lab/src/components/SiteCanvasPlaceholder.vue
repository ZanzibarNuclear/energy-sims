<script setup lang="ts">
/**
 * Clean-slate site canvas shell.
 * Lab-PR2 will replace this with place-intake / penstock / turbine tools.
 */
</script>

<template>
  <section class="canvas" aria-label="Site canvas">
    <header class="toolbar" aria-label="Construction tools (coming soon)">
      <span class="tool disabled" title="Lab-PR2">Intake</span>
      <span class="tool disabled" title="Lab-PR2">Penstock</span>
      <span class="tool disabled" title="Lab-PR2">Turbine</span>
      <span class="tool disabled" title="Lab-PR2">Select</span>
    </header>

    <div class="plot">
      <div class="axes" aria-hidden="true">
        <span class="y-label">elevation (m)</span>
        <span class="x-label">distance along ground (m)</span>
      </div>
      <div class="empty">
        <p class="title">Clean slate</p>
        <p>
          Place an <strong>intake</strong>, lay the <strong>penstock</strong>, and position the
          <strong>turbine</strong>. Construction tools arrive in the next lab PR.
        </p>
        <p class="muted">
          x → ground distance · y → elevation · export will target the production engine
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.canvas {
  display: flex;
  flex-direction: column;
  min-height: 22rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  overflow: hidden;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  padding: 0.55rem 0.7rem;
  border-bottom: 1px solid var(--border);
  background: var(--toolbar);
}

.tool {
  font-size: 0.8rem;
  padding: 0.3rem 0.65rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--muted-fg);
}

.tool.disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.plot {
  position: relative;
  flex: 1;
  min-height: 18rem;
  background:
    linear-gradient(to right, transparent 0, transparent calc(100% - 1px), var(--grid-line) 100%),
    linear-gradient(to bottom, transparent 0, transparent calc(100% - 1px), var(--grid-line) 100%),
    repeating-linear-gradient(
      to right,
      transparent,
      transparent 39px,
      var(--grid-line) 39px,
      var(--grid-line) 40px
    ),
    repeating-linear-gradient(
      to bottom,
      transparent,
      transparent 39px,
      var(--grid-line) 39px,
      var(--grid-line) 40px
    ),
    var(--canvas-bg);
}

.axes {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.y-label {
  position: absolute;
  left: 0.5rem;
  top: 50%;
  transform: translateY(-50%) rotate(-90deg);
  transform-origin: left center;
  font-size: 0.7rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted-fg);
  white-space: nowrap;
}

.x-label {
  position: absolute;
  bottom: 0.45rem;
  left: 50%;
  transform: translateX(-50%);
  font-size: 0.7rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted-fg);
}

.empty {
  position: absolute;
  inset: 2.5rem 2rem 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.5rem;
  padding: 1.25rem;
  border: 1px dashed var(--border-strong);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel) 88%, transparent);
  max-width: 28rem;
  margin: auto;
}

.title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 650;
}

.empty p {
  margin: 0;
  max-width: 24rem;
  line-height: 1.45;
  color: var(--fg);
  font-size: 0.92rem;
}

.muted {
  color: var(--muted-fg) !important;
  font-size: 0.8rem !important;
}
</style>
