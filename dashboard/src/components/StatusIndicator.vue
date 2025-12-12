<script setup lang="ts">
defineProps<{
  status: 'connected' | 'disconnected' | 'connecting' | 'error' | 'running' | 'stopped'
  label?: string
  size?: 'sm' | 'md' | 'lg'
}>()

const getStatusColor = (status: string) => {
  switch (status) {
    case 'connected':
    case 'running':
      return 'bg-emerald-500'
    case 'connecting':
      return 'bg-amber-500'
    case 'disconnected':
    case 'stopped':
      return 'bg-slate-500'
    case 'error':
      return 'bg-red-500'
    default:
      return 'bg-slate-500'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'connected': return 'Connected'
    case 'disconnected': return 'Disconnected'
    case 'connecting': return 'Connecting...'
    case 'error': return 'Error'
    case 'running': return 'Running'
    case 'stopped': return 'Stopped'
    default: return status
  }
}
</script>

<template>
  <div class="flex items-center gap-2">
    <span 
      class="relative flex rounded-full"
      :class="{
        'h-2 w-2': size === 'sm',
        'h-3 w-3': size === 'md' || !size,
        'h-4 w-4': size === 'lg'
      }"
    >
      <span 
        v-if="status === 'connected' || status === 'running'"
        class="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
        :class="getStatusColor(status)"
      ></span>
      <span 
        class="relative inline-flex rounded-full h-full w-full"
        :class="getStatusColor(status)"
      ></span>
    </span>
    <span v-if="label || status" class="text-sm" :class="{
      'text-emerald-400': status === 'connected' || status === 'running',
      'text-amber-400': status === 'connecting',
      'text-slate-400': status === 'disconnected' || status === 'stopped',
      'text-red-400': status === 'error'
    }">
      {{ label || getStatusText(status) }}
    </span>
  </div>
</template>

<style scoped></style>
