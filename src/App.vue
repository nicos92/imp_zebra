<script setup lang="ts">
const navItems = [
  { to: "/", label: "Panel" },
  { to: "/settings", label: "Configuración" },
  { to: "/history", label: "Historial" },
];
</script>

<template>
  <div class="app-shell">
    <header class="app-header">
      <h1 class="app-title">Zebra Printer</h1>
      <nav class="app-nav">
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="nav-link"
          active-class="active"
        >
          {{ item.label }}
        </RouterLink>
      </nav>
    </header>
    <main class="app-main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.app-title {
  margin: 0;
  font-size: 1.25rem;
}

.app-nav {
  display: flex;
  gap: 0.5rem;
}

.nav-link {
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius);
  color: var(--color-text);
  text-decoration: none;
}

.nav-link:hover {
  background: var(--color-border);
}

.nav-link.active {
  background: var(--color-primary);
  color: #fff;
}

.app-main {
  flex: 1;
  padding: 1.5rem;
}
</style>
