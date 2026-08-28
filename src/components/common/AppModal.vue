<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import AppButton from "./AppButton.vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    title?: string;
    closable?: boolean;
  }>(),
  {
    title: "",
    closable: true,
  },
);

const emit = defineEmits<{
  close: [];
}>();

function onKeydown(event: KeyboardEvent): void {
  if (props.open && props.closable && event.key === "Escape") {
    emit("close");
  }
}

onMounted(() => window.addEventListener("keydown", onKeydown));
onUnmounted(() => window.removeEventListener("keydown", onKeydown));
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="app-modal"
      role="dialog"
      aria-modal="true"
      @click.self="closable && emit('close')"
    >
      <div class="app-modal__dialog">
        <header class="app-modal__header">
          <h3 v-if="title" class="app-modal__title">{{ title }}</h3>
          <slot name="header" />
          <AppButton
            v-if="closable"
            class="app-modal__close"
            variant="secondary"
            aria-label="Cerrar"
            @click="emit('close')"
          >
            ×
          </AppButton>
        </header>

        <div class="app-modal__body">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.app-modal {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  background: rgb(0 0 0 / 50%);
  z-index: 100;
}

.app-modal__dialog {
  width: 100%;
  max-width: 480px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.app-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;
  border-bottom: 1px solid var(--color-border);
}

.app-modal__title {
  margin: 0;
  font-size: 1rem;
}

.app-modal__close {
  padding: 0.25rem 0.625rem;
}

.app-modal__body {
  padding: 1rem;
}
</style>
