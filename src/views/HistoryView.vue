<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getPrintJob, listPrintJobs } from "../infrastructure/tauri/printingApi";
import { commandErrorMessage } from "../infrastructure/tauri/tauriClient";
import type { PrintJob } from "../types";
import { formatDate, formatStatus } from "../utils/format";
import AppButton from "../components/common/AppButton.vue";

const jobs = ref<PrintJob[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const selectedJob = ref<PrintJob | null>(null);
const detailLoading = ref(false);
const detailError = ref<string | null>(null);

async function load(): Promise<void> {
  loading.value = true;
  error.value = null;
  try {
    jobs.value = await listPrintJobs(50);
    selectedJob.value = null;
    detailError.value = null;
  } catch (e) {
    error.value = commandErrorMessage(e);
  } finally {
    loading.value = false;
  }
}

async function selectJob(job: PrintJob): Promise<void> {
  if (detailLoading.value) return;
  detailLoading.value = true;
  detailError.value = null;
  selectedJob.value = job;
  try {
    selectedJob.value = await getPrintJob(job.id);
  } catch (e) {
    detailError.value = commandErrorMessage(e);
  } finally {
    detailLoading.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="history">
    <div class="history__header">
      <h2 class="history__title">Historial de trabajos</h2>
      <AppButton variant="secondary" @click="load">Actualizar</AppButton>
    </div>

    <p v-if="loading" class="history__muted">Cargando historial...</p>
    <p v-if="error" class="history__error" role="alert">{{ error }}</p>

    <div v-else-if="jobs.length === 0" class="history__empty">
      No hay trabajos de impresión registrados.
    </div>

    <table v-else class="history__table">
      <thead>
        <tr>
          <th>Fecha</th>
          <th>Cantidad</th>
          <th>Código inicial</th>
          <th>Código final</th>
          <th>Estado</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="job in jobs" :key="job.id">
          <td>{{ formatDate(job.created_at) }}</td>
          <td>{{ job.quantity }}</td>
          <td>
            <code>{{ job.start_code }}</code>
          </td>
          <td>
            <code>{{ job.end_code }}</code>
          </td>
          <td>
            <span class="history__status" :class="`history__status--${job.status}`">
              {{ formatStatus(job.status) }}
            </span>
          </td>
          <td>
            <AppButton variant="secondary" @click="selectJob(job)">
              Detalle
            </AppButton>
          </td>
        </tr>
      </tbody>
    </table>

    <section v-if="selectedJob || detailLoading" class="history__detail" aria-live="polite">
      <h3 class="history__detail-title">Detalle del trabajo</h3>
      <p v-if="detailLoading" class="history__muted">Cargando detalle...</p>
      <p v-else-if="detailError" class="history__error" role="alert">{{ detailError }}</p>
      <dl v-else-if="selectedJob" class="history__detail-list">
        <div class="history__detail-row">
          <dt>ID</dt>
          <dd><code>{{ selectedJob.id }}</code></dd>
        </div>
        <div class="history__detail-row">
          <dt>Printer ID</dt>
          <dd><code>{{ selectedJob.printer_id }}</code></dd>
        </div>
        <div class="history__detail-row">
          <dt>Cantidad</dt>
          <dd>{{ selectedJob.quantity }}</dd>
        </div>
        <div class="history__detail-row">
          <dt>Código inicial</dt>
          <dd><code>{{ selectedJob.start_code }}</code></dd>
        </div>
        <div class="history__detail-row">
          <dt>Código final</dt>
          <dd><code>{{ selectedJob.end_code }}</code></dd>
        </div>
        <div class="history__detail-row">
          <dt>Estado</dt>
          <dd>{{ formatStatus(selectedJob.status) }}</dd>
        </div>
        <div class="history__detail-row">
          <dt>Creado</dt>
          <dd>{{ formatDate(selectedJob.created_at) }}</dd>
        </div>
        <div class="history__detail-row">
          <dt>Completado</dt>
          <dd>{{ selectedJob.completed_at ? formatDate(selectedJob.completed_at) : "—" }}</dd>
        </div>
      </dl>
    </section>
  </div>
</template>

<style scoped>
.history {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.history__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.history__title {
  margin: 0;
  font-size: 1.25rem;
}

.history__muted {
  color: var(--color-muted);
}

.history__error {
  color: var(--color-danger);
}

.history__empty {
  padding: 1rem;
  border: 1px dashed var(--color-border);
  border-radius: var(--radius);
  color: var(--color-muted);
}

.history__table {
  width: 100%;
  border-collapse: collapse;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.history__table th,
.history__table td {
  padding: 0.625rem 0.875rem;
  border-bottom: 1px solid var(--color-border);
  text-align: left;
  font-size: 0.875rem;
}

.history__table th {
  background: var(--color-border);
  font-weight: 600;
}

.history__status {
  padding: 0.125rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  color: #fff;
}

.history__status--completed {
  background: var(--color-success);
}

.history__status--pending {
  background: var(--color-warning);
}

.history__status--printing {
  background: var(--color-primary);
}

.history__status--failed {
  background: var(--color-danger);
}

.history__detail {
  border: 1px solid var(--color-border);
  border-radius: var(--radius);
  background: var(--color-surface);
  box-shadow: var(--shadow);
  padding: 1rem;
}

.history__detail-title {
  margin: 0 0 0.75rem;
  font-size: 1rem;
}

.history__detail-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 0;
}

.history__detail-row {
  display: flex;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.history__detail-row dt {
  width: 8rem;
  flex-shrink: 0;
  color: var(--color-muted);
  font-weight: 600;
}

.history__detail-row dd {
  margin: 0;
}
</style>
