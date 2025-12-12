<script setup lang="ts">
import Modal from './Modal.vue'

defineProps<{
  show: boolean
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  type?: 'danger' | 'warning' | 'info'
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <Modal :show="show" :title="title || 'Confirm'" size="sm" @close="emit('cancel')">
    <div class="flex items-start gap-4">
      <!-- Icon -->
      <div 
        class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center"
        :class="{
          'bg-red-500/20': type === 'danger',
          'bg-amber-500/20': type === 'warning',
          'bg-cyan-500/20': type === 'info' || !type
        }"
      >
        <svg 
          v-if="type === 'danger'" 
          class="w-5 h-5 text-red-400" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
        <svg 
          v-else-if="type === 'warning'" 
          class="w-5 h-5 text-amber-400" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <svg 
          v-else 
          class="w-5 h-5 text-cyan-400" 
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      </div>

      <!-- Message -->
      <p class="text-slate-300 text-sm leading-relaxed flex-1">{{ message }}</p>
    </div>

    <template #footer>
      <button @click="emit('cancel')" class="btn-secondary">
        {{ cancelText || 'Cancel' }}
      </button>
      <button 
        @click="emit('confirm')" 
        :class="type === 'danger' ? 'btn-danger' : 'btn-primary'"
      >
        {{ confirmText || 'Confirm' }}
      </button>
    </template>
  </Modal>
</template>

<style scoped></style>
