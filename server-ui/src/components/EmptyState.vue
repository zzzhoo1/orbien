<script setup lang="ts">
defineProps<{
  type?: 'clients' | 'tunnels' | 'search' | 'filter' | 'generic'
  title: string
  desc?: string
}>()
</script>

<template>
  <div class="empty-state" role="status" aria-live="polite">
    <div class="empty-illustration" aria-hidden="true">
      <!-- clients -->
      <svg v-if="type === 'clients'" viewBox="0 0 80 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="8" y="20" width="28" height="34" rx="4" fill="currentColor" opacity="0.07"/>
        <rect x="44" y="20" width="28" height="34" rx="4" fill="currentColor" opacity="0.07"/>
        <circle cx="22" cy="13" r="7" fill="currentColor" opacity="0.15"/>
        <circle cx="58" cy="13" r="7" fill="currentColor" opacity="0.15"/>
        <rect x="15" y="30" width="14" height="2.5" rx="1.25" fill="currentColor" opacity="0.2"/>
        <rect x="15" y="36" width="10" height="2" rx="1" fill="currentColor" opacity="0.15"/>
        <rect x="51" y="30" width="14" height="2.5" rx="1.25" fill="currentColor" opacity="0.2"/>
        <rect x="51" y="36" width="10" height="2" rx="1" fill="currentColor" opacity="0.15"/>
        <path d="M36 37 Q40 32 44 37" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.2" fill="none"/>
      </svg>

      <!-- tunnels -->
      <svg v-else-if="type === 'tunnels'" viewBox="0 0 80 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="6" y="26" width="20" height="12" rx="3" fill="currentColor" opacity="0.12"/>
        <rect x="54" y="26" width="20" height="12" rx="3" fill="currentColor" opacity="0.12"/>
        <path d="M26 32 L54 32" stroke="currentColor" stroke-width="2" stroke-dasharray="4 3" stroke-linecap="round" opacity="0.18"/>
        <circle cx="40" cy="32" r="5" fill="currentColor" opacity="0.1"/>
        <path d="M37.5 32 L39 33.5 L42.5 30" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" opacity="0.25"/>
        <rect x="10" y="30" width="12" height="2" rx="1" fill="currentColor" opacity="0.2"/>
        <rect x="58" y="30" width="12" height="2" rx="1" fill="currentColor" opacity="0.2"/>
        <circle cx="40" cy="14" r="4" fill="currentColor" opacity="0.08"/>
        <circle cx="40" cy="50" r="4" fill="currentColor" opacity="0.08"/>
      </svg>

      <!-- search empty -->
      <svg v-else-if="type === 'search'" viewBox="0 0 80 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="34" cy="30" r="16" stroke="currentColor" stroke-width="2.5" opacity="0.15"/>
        <path d="M46 42 L58 54" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" opacity="0.18"/>
        <path d="M28 30 L32 34 L40 26" stroke="currentColor" stroke-width="0" fill="none"/>
        <line x1="29" y1="30" x2="39" y2="30" stroke="currentColor" stroke-width="2" stroke-linecap="round" opacity="0.2"/>
        <line x1="34" y1="25" x2="34" y2="35" stroke="currentColor" stroke-width="2" stroke-linecap="round" opacity="0.2"/>
        <circle cx="34" cy="30" r="3" fill="currentColor" opacity="0.08"/>
      </svg>

      <!-- filter empty -->
      <svg v-else-if="type === 'filter'" viewBox="0 0 80 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M16 20 H64 L50 36 V52 L30 44 V36 Z" fill="currentColor" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" opacity="0.15"/>
        <rect x="28" y="40" width="24" height="2" rx="1" fill="currentColor" opacity="0.12"/>
        <circle cx="60" cy="48" r="8" fill="currentColor" opacity="0.07"/>
        <line x1="57" y1="48" x2="63" y2="48" stroke="currentColor" stroke-width="2" stroke-linecap="round" opacity="0.2"/>
      </svg>

      <!-- generic -->
      <svg v-else viewBox="0 0 80 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect x="16" y="18" width="48" height="36" rx="5" fill="currentColor" opacity="0.07"/>
        <rect x="24" y="26" width="32" height="3" rx="1.5" fill="currentColor" opacity="0.18"/>
        <rect x="24" y="33" width="22" height="2.5" rx="1.25" fill="currentColor" opacity="0.13"/>
        <rect x="24" y="39" width="16" height="2" rx="1" fill="currentColor" opacity="0.1"/>
        <circle cx="40" cy="10" r="4" fill="currentColor" opacity="0.1"/>
      </svg>
    </div>

    <p class="empty-title">{{ title }}</p>
    <p v-if="desc" class="empty-desc">{{ desc }}</p>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.55rem;
  padding: 2.75rem 1.5rem 2.5rem;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  color: var(--muted);
  text-align: center;
  animation: empty-fade-in 0.35s cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes empty-fade-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .empty-state {
    animation: none;
  }
}

.empty-illustration {
  width: 5rem;
  height: 4rem;
  color: var(--muted);
  flex-shrink: 0;
}

.empty-illustration svg {
  width: 100%;
  height: 100%;
  display: block;
}

.empty-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--muted);
  margin: 0;
  max-width: 28ch;
}

.empty-desc {
  font-size: 0.78rem;
  color: var(--muted);
  opacity: 0.7;
  margin: 0;
  max-width: 36ch;
  line-height: 1.5;
}
</style>
