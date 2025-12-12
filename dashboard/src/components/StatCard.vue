<script setup lang="ts">
defineProps<{
  title: string
  value: string | number
  subtitle?: string
  trend?: 'up' | 'down' | 'neutral'
  trendValue?: string
  icon?: string
  color?: 'cyan' | 'purple' | 'green' | 'amber' | 'red'
}>()
</script>

<template>
  <div class="glass-card p-6 transition-smooth hover:border-slate-600">
    <div class="flex items-start justify-between">
      <div class="flex-1">
        <p class="text-sm text-slate-400 font-medium">{{ title }}</p>
        <p class="text-3xl font-bold mt-2" :class="{
          'text-cyan-400': color === 'cyan' || !color,
          'text-purple-400': color === 'purple',
          'text-emerald-400': color === 'green',
          'text-amber-400': color === 'amber',
          'text-red-400': color === 'red'
        }">
          {{ value }}
        </p>
        <p v-if="subtitle" class="text-sm text-slate-500 mt-1">{{ subtitle }}</p>
        
        <!-- Trend indicator -->
        <div v-if="trend" class="flex items-center gap-1 mt-2">
          <svg v-if="trend === 'up'" class="w-4 h-4 text-emerald-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M5.293 9.707a1 1 0 010-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 01-1.414 1.414L11 7.414V15a1 1 0 11-2 0V7.414L6.707 9.707a1 1 0 01-1.414 0z" clip-rule="evenodd" />
          </svg>
          <svg v-if="trend === 'down'" class="w-4 h-4 text-red-400" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M14.707 10.293a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 111.414-1.414L9 12.586V5a1 1 0 012 0v7.586l2.293-2.293a1 1 0 011.414 0z" clip-rule="evenodd" />
          </svg>
          <span class="text-sm" :class="trend === 'up' ? 'text-emerald-400' : trend === 'down' ? 'text-red-400' : 'text-slate-400'">
            {{ trendValue }}
          </span>
        </div>
      </div>
      
      <!-- Icon -->
      <div class="w-12 h-12 rounded-xl flex items-center justify-center" :class="{
        'bg-cyan-500/20': color === 'cyan' || !color,
        'bg-purple-500/20': color === 'purple',
        'bg-emerald-500/20': color === 'green',
        'bg-amber-500/20': color === 'amber',
        'bg-red-500/20': color === 'red'
      }">
        <slot name="icon">
          <svg class="w-6 h-6" :class="{
            'text-cyan-400': color === 'cyan' || !color,
            'text-purple-400': color === 'purple',
            'text-emerald-400': color === 'green',
            'text-amber-400': color === 'amber',
            'text-red-400': color === 'red'
          }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
        </slot>
      </div>
    </div>
  </div>
</template>

<style scoped></style>
