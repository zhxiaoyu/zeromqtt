<script setup lang="ts">
import Sidebar from '@/components/Sidebar.vue'
</script>

<template>
  <div class="flex min-h-screen">
    <!-- Sidebar -->
    <Sidebar />

    <!-- Main content area -->
    <div class="flex-1 flex flex-col ml-64">
      <!-- Top bar -->
      <header class="h-16 bg-slate-800/50 backdrop-blur-md border-b border-slate-700/50 flex items-center justify-between px-6 sticky top-0 z-10">
        <div class="flex items-center gap-4">
          <slot name="header-left">
            <h1 class="text-xl font-semibold text-white">
              <slot name="title">Dashboard</slot>
            </h1>
          </slot>
        </div>
        <div class="flex items-center gap-4">
          <slot name="header-right" />
        </div>
      </header>

      <!-- Page content -->
      <main class="flex-1 p-6 overflow-auto">
        <slot />
      </main>
    </div>
  </div>
</template>

<style scoped></style>
