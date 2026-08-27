<script setup lang="ts">
import SectionCard from '@/components/SectionCard.vue'
import StatusBadge, {type StatusSize, type StatusType} from '@/components/StatusBadge.vue'

interface Scenario {
  name: string
  status: StatusType
  label?: string
  size?: StatusSize
  dot?: boolean
}

const statusScenarios: Scenario[] = [
  {name: 'Online', status: 'online', label: 'Online'},
  {name: 'Offline', status: 'offline', label: 'Offline'},
  {name: 'Running', status: 'running', label: 'Running', dot: true},
  {name: 'Stopped', status: 'stopped', label: 'Stopped', dot: true},
  {name: 'Pending', status: 'pending', label: 'Pending', dot: true},
  {name: 'Error', status: 'error', label: 'Error', dot: true},
  {name: 'Info', status: 'info', label: 'Info', dot: true},
]

const contentScenarios: Scenario[] = [
  {name: 'No label', status: 'info'},
  {name: 'Small size', status: 'online', label: 'Online', size: 'sm', dot: true},
  {name: 'Large number', status: 'running', label: '9,223,372,036,854,775,807'},
  {name: 'Long words', status: 'pending', label: 'Synchronizing the remote connection configuration'},
  {name: 'Unbreakable token', status: 'error', label: 'a4f9d6b3e7c81a25f0d4b9e6c2a8d1f7b5e3c9a6'},
]
</script>

<template>
  <main class="status-badge-test">
    <header class="page-head">
      <p class="eyebrow">Development-only component check</p>
      <h1>StatusBadge stress cases</h1>
      <p class="intro">All cases use the production component and the application theme tokens.</p>
    </header>

    <SectionCard title="Supported status values">
      <div class="scenario-grid">
        <div v-for="scenario in statusScenarios" :key="scenario.name" class="scenario">
          <span class="scenario-name">{{ scenario.name }}</span>
          <StatusBadge
              :status="scenario.status"
              :label="scenario.label"
              :size="scenario.size"
              :dot="scenario.dot"
          />
        </div>
      </div>
    </SectionCard>

    <SectionCard title="Content and size extremes">
      <div class="scenario-grid">
        <div v-for="scenario in contentScenarios" :key="scenario.name" class="scenario">
          <span class="scenario-name">{{ scenario.name }}</span>
          <StatusBadge
              :status="scenario.status"
              :label="scenario.label"
              :size="scenario.size"
              :dot="scenario.dot"
          />
        </div>
      </div>
    </SectionCard>
  </main>
</template>

<style scoped>
.status-badge-test {
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  min-width: 0;
  animation: page-in 0.35s ease both;
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-head { display: flex; flex-direction: column; gap: 0.3rem; }
.eyebrow { margin: 0; color: var(--accent-text); font-size: 0.75rem; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; }
h1 { margin: 0; color: var(--text); font-size: 1.35rem; line-height: 1.2; }
.intro { margin: 0; color: var(--muted); font-size: 0.86rem; }

.scenario-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
  gap: 0.75rem;
}

.scenario {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.45rem;
  min-width: 0;
  padding: 0.8rem;
  border: 1px solid var(--line);
  border-radius: var(--radius-md);
  background: color-mix(in srgb, var(--muted) 4%, transparent);
}

.scenario-name { color: var(--muted); font-size: 0.76rem; font-weight: 600; }

@media (max-width: 400px) {
  .scenario-grid { grid-template-columns: minmax(0, 1fr); }
}
</style>
