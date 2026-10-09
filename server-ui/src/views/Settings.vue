<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { fetchSystemHealth, fetchSystemInfo, reloadConfig } from '@/api'
import type { HealthInfo, SystemInfo } from '@/types/api'

const { t } = useI18n()

// ── state ──────────────────────────────────────────────────────────────

const health = ref<HealthInfo | null>(null)
const sysInfo = ref<SystemInfo | null>(null)
const healthLoading = ref(false)
const healthError = ref('')

const reloading = ref(false)
const reloadResult = ref<{ changed: string[]; ts: string } | null>(null)
const reloadError = ref('')

// ── lifecycle ───────────────────────────────────────────────────────────

onMounted(async () => {
    await refreshHealth()
})

// ── actions ─────────────────────────────────────────────────────────────

async function refreshHealth() {
    healthLoading.value = true
    healthError.value = ''
    try {
        const [h, s] = await Promise.all([fetchSystemHealth(), fetchSystemInfo()])
        health.value = h
        sysInfo.value = s
    } catch (e: any) {
        healthError.value = e?.message ?? t('settings.healthFetchFailed')
    } finally {
        healthLoading.value = false
    }
}

async function doReload() {
    reloading.value = true
    reloadError.value = ''
    reloadResult.value = null
    try {
        const resp = await reloadConfig()
        reloadResult.value = {
            changed: resp.changed,
            ts: new Date().toLocaleTimeString(),
        }
    } catch (e: any) {
        reloadError.value = e?.message ?? t('settings.reloadFailed')
    } finally {
        reloading.value = false
    }
}

function formatUptime(secs: number): string {
    if (secs < 60) return `${secs}s`
    if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`
    const h = Math.floor(secs / 3600)
    const m = Math.floor((secs % 3600) / 60)
    return `${h}h ${m}m`
}
</script>

<template>
    <div class="settings-page">
        <h1 class="page-title">{{ t('nav.settings') }}</h1>

        <!-- ── Health card ──────────────────────────────────────────── -->
        <section class="card">
            <div class="card-header">
                <h2>{{ t('settings.healthTitle') }}</h2>
                <button class="btn-ghost" :disabled="healthLoading" @click="refreshHealth">
                    <svg class="icon" :class="{ spinning: healthLoading }" viewBox="0 0 24 24" fill="none"
                         stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="23 4 23 10 17 10"/>
                        <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
                    </svg>
                    {{ t('common.refresh') }}
                </button>
            </div>

            <p v-if="healthError" class="error-text">{{ healthError }}</p>

            <div v-else-if="health" class="health-grid">
                <!-- status badge -->
                <div class="health-item">
                    <span class="label">{{ t('settings.status') }}</span>
                    <span class="badge" :class="health.status === 'ok' ? 'badge-ok' : 'badge-err'">
                        {{ health.status }}
                    </span>
                </div>
                <div class="health-item">
                    <span class="label">{{ t('settings.version') }}</span>
                    <span class="value mono">{{ health.version }}</span>
                </div>
                <div class="health-item">
                    <span class="label">{{ t('settings.uptime') }}</span>
                    <span class="value">{{ formatUptime(health.uptimeSecs) }}</span>
                </div>
                <div class="health-item">
                    <span class="label">{{ t('settings.onlineClients') }}</span>
                    <span class="value">{{ health.onlineClients }}</span>
                </div>
                <div class="health-item">
                    <span class="label">{{ t('settings.activeConns') }}</span>
                    <span class="value">{{ health.activeConnections }}</span>
                </div>
            </div>

            <div v-else-if="healthLoading" class="skeleton-group">
                <div v-for="i in 5" :key="i" class="skeleton-row" />
            </div>
        </section>

        <!-- ── Server config (read-only) ──────────────────────────────── -->
        <section v-if="sysInfo" class="card">
            <div class="card-header">
                <h2>{{ t('settings.configTitle') }}</h2>
            </div>
            <div class="config-grid">
                <template v-for="(val, key) in sysInfo.config" :key="key">
                    <span class="label">{{ key }}</span>
                    <span class="value mono">{{ val ?? '—' }}</span>
                </template>
            </div>
        </section>

        <!-- ── Reload config ──────────────────────────────────────── -->
        <section class="card">
            <div class="card-header">
                <h2>{{ t('settings.reloadTitle') }}</h2>
            </div>
            <p class="muted-text">{{ t('settings.reloadDesc') }}</p>

            <button class="btn-primary" :disabled="reloading" @click="doReload">
                <svg v-if="reloading" class="icon spinning" viewBox="0 0 24 24" fill="none"
                     stroke="currentColor" stroke-width="2">
                    <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                </svg>
                {{ reloading ? t('settings.reloading') : t('settings.reloadBtn') }}
            </button>

            <div v-if="reloadResult" class="reload-result ok">
                <strong>{{ t('settings.reloadSuccess') }}</strong>
                <span class="ts">{{ reloadResult.ts }}</span>
                <template v-if="reloadResult.changed.length">
                    <p class="changed-label">{{ t('settings.changedKeys') }}</p>
                    <ul class="changed-list">
                        <li v-for="k in reloadResult.changed" :key="k" class="mono">{{ k }}</li>
                    </ul>
                </template>
                <p v-else class="muted-text">{{ t('settings.noChanges') }}</p>
            </div>

            <p v-if="reloadError" class="error-text">{{ reloadError }}</p>
        </section>
    </div>
</template>

<style scoped>
.settings-page {
    padding: 2rem;
    max-width: 760px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
}

.page-title {
    font-size: 1.4rem;
    font-weight: 600;
    color: var(--color-text, #1a1a1a);
}

/* card */
.card {
    background: var(--color-surface, #fff);
    border: 1px solid var(--color-border, #e5e7eb);
    border-radius: 0.625rem;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.card-header h2 {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text, #111);
}

/* health grid */
.health-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 0.75rem 1rem;
}

.health-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

/* config grid */
.config-grid {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 0.4rem 1.5rem;
    font-size: 0.875rem;
}

.label {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--color-text-muted, #6b7280);
    text-transform: uppercase;
    letter-spacing: 0.04em;
}

.value {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-text, #111);
}

.mono {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.82rem;
}

/* badge */
.badge {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.55rem;
    border-radius: 9999px;
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    width: fit-content;
}

.badge-ok {
    background: color-mix(in srgb, #22c55e 15%, transparent);
    color: #16a34a;
}

.badge-err {
    background: color-mix(in srgb, #ef4444 15%, transparent);
    color: #dc2626;
}

/* buttons */
.btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1.1rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    background: var(--color-primary, #01696f);
    color: #fff;
    transition: background 150ms;
    align-self: flex-start;
}
.btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover, #0c4e54);
}
.btn-primary:disabled {
    opacity: 0.55;
    cursor: not-allowed;
}

.btn-ghost {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.75rem;
    border-radius: 0.45rem;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid var(--color-border, #e5e7eb);
    background: transparent;
    color: var(--color-text-muted, #6b7280);
    transition: background 150ms, color 150ms;
}
.btn-ghost:hover:not(:disabled) {
    background: var(--color-surface-offset, #f3f4f6);
    color: var(--color-text, #111);
}
.btn-ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* icon */
.icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
}

@keyframes spin { to { transform: rotate(360deg); } }
.spinning { animation: spin 0.75s linear infinite; }

/* reload result */
.reload-result {
    padding: 0.9rem 1rem;
    border-radius: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.875rem;
}
.reload-result.ok {
    background: color-mix(in srgb, #22c55e 10%, transparent);
    border: 1px solid color-mix(in srgb, #22c55e 30%, transparent);
    color: var(--color-text, #111);
}
.reload-result .ts {
    font-size: 0.75rem;
    color: var(--color-text-muted, #6b7280);
}
.changed-label {
    font-weight: 600;
    margin-top: 0.25rem;
}
.changed-list {
    list-style: disc;
    padding-left: 1.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
}

/* error */
.error-text {
    font-size: 0.85rem;
    color: var(--color-error, #dc2626);
}

/* muted */
.muted-text {
    font-size: 0.85rem;
    color: var(--color-text-muted, #6b7280);
}

/* skeleton */
@keyframes shimmer {
    0%   { background-position: -200% 0; }
    100% { background-position:  200% 0; }
}
.skeleton-group { display: flex; flex-direction: column; gap: 0.6rem; }
.skeleton-row {
    height: 1.1rem;
    border-radius: 0.3rem;
    background: linear-gradient(90deg,
        var(--color-surface-offset, #f3f4f6) 25%,
        var(--color-surface-dynamic, #e5e7eb) 50%,
        var(--color-surface-offset, #f3f4f6) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.4s ease-in-out infinite;
}
.skeleton-row:nth-child(2) { width: 85%; }
.skeleton-row:nth-child(3) { width: 70%; }
.skeleton-row:nth-child(4) { width: 90%; }
.skeleton-row:nth-child(5) { width: 60%; }

@media (max-width: 640px) {
    .settings-page { padding: 1rem; }
    .config-grid { grid-template-columns: 1fr; }
}
</style>
