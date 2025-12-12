<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const username = ref('')
const password = ref('')
const isLoading = ref(false)
const errorMessage = ref('')

const handleLogin = async () => {
  if (!username.value || !password.value) {
    errorMessage.value = 'Please enter username and password'
    return
  }

  isLoading.value = true
  errorMessage.value = ''

  const success = await authStore.login(username.value, password.value)
  
  if (success) {
    router.push('/')
  } else {
    errorMessage.value = authStore.error || 'Login failed'
  }
  
  isLoading.value = false
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-4">
    <!-- Background decoration -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none">
      <div class="absolute top-1/4 left-1/4 w-96 h-96 bg-cyan-500/10 rounded-full blur-3xl"></div>
      <div class="absolute bottom-1/4 right-1/4 w-96 h-96 bg-purple-500/10 rounded-full blur-3xl"></div>
    </div>

    <!-- Login card -->
    <div class="glass-card p-8 w-full max-w-md relative z-10">
      <!-- Logo -->
      <div class="flex flex-col items-center mb-8">
        <div class="w-16 h-16 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-2xl flex items-center justify-center mb-4">
          <svg class="w-10 h-10 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <h1 class="text-2xl font-bold gradient-text">ZeroMQTT</h1>
        <p class="text-slate-400 mt-1">Bridge Management Dashboard</p>
      </div>

      <!-- Login form -->
      <form @submit.prevent="handleLogin" class="space-y-6">
        <!-- Error message -->
        <div v-if="errorMessage" class="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p class="text-red-400 text-sm">{{ errorMessage }}</p>
        </div>

        <!-- Username -->
        <div>
          <label for="username" class="block text-sm font-medium text-slate-300 mb-2">
            Username
          </label>
          <input
            id="username"
            v-model="username"
            type="text"
            placeholder="Enter username"
            class="input-dark w-full"
            autocomplete="username"
          />
        </div>

        <!-- Password -->
        <div>
          <label for="password" class="block text-sm font-medium text-slate-300 mb-2">
            Password
          </label>
          <input
            id="password"
            v-model="password"
            type="password"
            placeholder="Enter password"
            class="input-dark w-full"
            autocomplete="current-password"
          />
        </div>

        <!-- Submit button -->
        <button
          type="submit"
          :disabled="isLoading"
          class="btn-primary w-full flex items-center justify-center gap-2"
          :class="{ 'opacity-50 cursor-not-allowed': isLoading }"
        >
          <svg v-if="isLoading" class="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <span>{{ isLoading ? 'Signing in...' : 'Sign In' }}</span>
        </button>

        <!-- Default credentials hint -->
        <p class="text-center text-sm text-slate-500">
          Default: <code class="text-cyan-400">zeromqtt</code> / <code class="text-cyan-400">zeromqtt</code>
        </p>
      </form>
    </div>
  </div>
</template>

<style scoped></style>
