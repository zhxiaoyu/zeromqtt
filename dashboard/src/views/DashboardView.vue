<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { Line } from 'vue-chartjs'
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler } from 'chart.js'
import MainLayout from '@/layouts/MainLayout.vue'
import StatCard from '@/components/StatCard.vue'
import StatusIndicator from '@/components/StatusIndicator.vue'
import { useBridgeStore } from '@/stores/bridge'
import { useAuthStore } from '@/stores/auth'

// Register Chart.js components
ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler)

const bridgeStore = useBridgeStore()
const authStore = useAuthStore()

let refreshInterval: ReturnType<typeof setInterval> | null = null

// Fetch data on mount
onMounted(async () => {
  await authStore.fetchUser()
  await Promise.all([
    bridgeStore.fetchStatus(),
    bridgeStore.fetchStats(),
    bridgeStore.fetchChartData(),
    bridgeStore.fetchMqttConfigs(),
    bridgeStore.fetchZmqConfigs()
  ])
  
  // Auto-refresh every 5 seconds
  refreshInterval = setInterval(async () => {
    await Promise.all([
      bridgeStore.fetchStatus(),
      bridgeStore.fetchStats()
    ])
  }, 5000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})

// Format uptime
const formatUptime = (seconds: number) => {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  
  if (hours > 0) {
    return `${hours}h ${minutes}m ${secs}s`
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`
  }
  return `${secs}s`
}

// Overall connection status based on enabled endpoints
const mqttConnectionStatus = computed(() => {
  if (!bridgeStore.status || bridgeStore.status.state !== 'running') return 'disconnected'
  const enabledBrokers = bridgeStore.mqttConfigs.filter(c => c.enabled)
  if (enabledBrokers.length === 0) return 'disconnected'
  return 'connected'
})

const zmqConnectionStatus = computed(() => {
  if (!bridgeStore.status || bridgeStore.status.state !== 'running') return 'disconnected'
  const enabledEndpoints = bridgeStore.zmqConfigs.filter(c => c.enabled)
  if (enabledEndpoints.length === 0) return 'disconnected'
  return 'connected'
})

// Chart data
const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'top' as const,
      labels: {
        color: '#94a3b8',
        usePointStyle: true
      }
    },
    tooltip: {
      backgroundColor: '#1e293b',
      titleColor: '#f1f5f9',
      bodyColor: '#94a3b8',
      borderColor: '#334155',
      borderWidth: 1
    }
  },
  scales: {
    x: {
      grid: {
        color: '#334155',
        drawBorder: false
      },
      ticks: {
        color: '#64748b'
      }
    },
    y: {
      grid: {
        color: '#334155',
        drawBorder: false
      },
      ticks: {
        color: '#64748b'
      }
    }
  },
  elements: {
    line: {
      tension: 0.4
    },
    point: {
      radius: 0,
      hitRadius: 10,
      hoverRadius: 4
    }
  }
}

const chartData = computed(() => {
  if (!bridgeStore.chartData.length) {
    return { labels: [], datasets: [] }
  }
  
  const mqttData = bridgeStore.chartData.find(d => d.label === 'MQTT')
  const zmqData = bridgeStore.chartData.find(d => d.label === 'ZeroMQ')
  
  const labels = mqttData?.data.map((_, i) => `${30 - i}m ago`).reverse() || []
  
  return {
    labels,
    datasets: [
      {
        label: 'MQTT',
        data: mqttData?.data.map(d => d.value).reverse() || [],
        borderColor: '#06b6d4',
        backgroundColor: 'rgba(6, 182, 212, 0.1)',
        fill: true
      },
      {
        label: 'ZeroMQ',
        data: zmqData?.data.map(d => d.value).reverse() || [],
        borderColor: '#8b5cf6',
        backgroundColor: 'rgba(139, 92, 246, 0.1)',
        fill: true
      }
    ]
  }
})
</script>

<template>
  <MainLayout>
    <template #title>Dashboard</template>
    
    <div class="space-y-6">
      <!-- Status header -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-2xl font-bold text-white">System Overview</h2>
          <p class="text-slate-400 mt-1">Real-time bridge status and performance metrics</p>
        </div>
        <div v-if="bridgeStore.status" class="flex items-center gap-4">
          <StatusIndicator :status="bridgeStore.status.state" size="lg" />
          <span class="text-sm text-slate-400">
            Version {{ bridgeStore.status.version }}
          </span>
        </div>
      </div>

      <!-- Stats grid -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Messages / sec"
          :value="bridgeStore.stats?.messages_per_second.toFixed(1) || '0'"
          subtitle="Current throughput"
          color="cyan"
        >
          <template #icon>
            <svg class="w-6 h-6 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </template>
        </StatCard>

        <StatCard
          title="Avg Latency"
          :value="`${bridgeStore.stats?.avg_latency_ms.toFixed(2) || '0'} ms`"
          subtitle="Message processing time"
          color="purple"
        >
          <template #icon>
            <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </template>
        </StatCard>

        <StatCard
          title="Queue Depth"
          :value="bridgeStore.stats?.queue_depth || 0"
          subtitle="Messages in queue"
          color="amber"
        >
          <template #icon>
            <svg class="w-6 h-6 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </template>
        </StatCard>

        <StatCard
          title="Uptime"
          :value="formatUptime(bridgeStore.status?.uptime_seconds || 0)"
          subtitle="Since last restart"
          color="green"
        >
          <template #icon>
            <svg class="w-6 h-6 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </template>
        </StatCard>
      </div>

      <!-- Connection status cards -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- MQTT Status -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <h3 class="text-lg font-semibold text-white">MQTT Brokers</h3>
              <span class="text-xs px-2 py-1 rounded bg-cyan-500/20 text-cyan-400">
                {{ bridgeStore.mqttConfigs.filter(c => c.enabled).length }} active
              </span>
            </div>
            <StatusIndicator :status="mqttConnectionStatus" />
          </div>
          
          <!-- Endpoint list -->
          <div v-if="bridgeStore.mqttConfigs.length" class="space-y-2 mb-4">
            <div 
              v-for="config in bridgeStore.mqttConfigs" 
              :key="config.id" 
              class="flex items-center justify-between p-2 rounded bg-slate-800/50"
            >
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full" :class="config.enabled ? 'bg-emerald-500' : 'bg-slate-500'"></div>
                <span class="text-sm text-slate-300">{{ config.name }}</span>
              </div>
              <span class="text-xs text-slate-500">{{ config.broker_url }}:{{ config.port }}</span>
            </div>
          </div>
          <div v-else class="text-sm text-slate-500 mb-4">No MQTT brokers configured</div>
          
          <div class="grid grid-cols-2 gap-4 pt-4 border-t border-slate-700/50">
            <div>
              <p class="text-sm text-slate-400">Received</p>
              <p class="text-2xl font-bold text-cyan-400">{{ bridgeStore.stats?.mqtt_received?.toLocaleString() || 0 }}</p>
            </div>
            <div>
              <p class="text-sm text-slate-400">Sent</p>
              <p class="text-2xl font-bold text-cyan-400">{{ bridgeStore.stats?.mqtt_sent?.toLocaleString() || 0 }}</p>
            </div>
          </div>
        </div>

        <!-- ZeroMQ Status -->
        <div class="glass-card p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <h3 class="text-lg font-semibold text-white">ZeroMQ Endpoints</h3>
              <span class="text-xs px-2 py-1 rounded bg-purple-500/20 text-purple-400">
                {{ bridgeStore.zmqConfigs.filter(c => c.enabled).length }} active
              </span>
            </div>
            <StatusIndicator :status="zmqConnectionStatus" />
          </div>
          
          <!-- Endpoint list -->
          <div v-if="bridgeStore.zmqConfigs.length" class="space-y-2 mb-4">
            <div 
              v-for="config in bridgeStore.zmqConfigs" 
              :key="config.id" 
              class="flex items-center justify-between p-2 rounded bg-slate-800/50"
            >
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full" :class="config.enabled ? 'bg-emerald-500' : 'bg-slate-500'"></div>
                <span class="text-sm text-slate-300">{{ config.name }}</span>
                <span class="text-xs px-1.5 py-0.5 rounded bg-purple-500/20 text-purple-400 uppercase">{{ config.socket_type }}</span>
              </div>
              <span class="text-xs text-slate-500">{{ config.bind_endpoint || 'No bind' }}</span>
            </div>
          </div>
          <div v-else class="text-sm text-slate-500 mb-4">No ZeroMQ endpoints configured</div>
          
          <div class="grid grid-cols-2 gap-4 pt-4 border-t border-slate-700/50">
            <div>
              <p class="text-sm text-slate-400">Received</p>
              <p class="text-2xl font-bold text-purple-400">{{ bridgeStore.stats?.zmq_received?.toLocaleString() || 0 }}</p>
            </div>
            <div>
              <p class="text-sm text-slate-400">Sent</p>
              <p class="text-2xl font-bold text-purple-400">{{ bridgeStore.stats?.zmq_sent?.toLocaleString() || 0 }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Throughput chart -->
      <div class="glass-card p-6">
        <h3 class="text-lg font-semibold text-white mb-4">Message Throughput (Last 30 minutes)</h3>
        <div class="h-64">
          <Line :data="chartData" :options="chartOptions" />
        </div>
      </div>

      <!-- Error count -->
      <div v-if="bridgeStore.stats?.error_count && bridgeStore.stats.error_count > 0" class="glass-card p-6 border-red-500/30">
        <div class="flex items-center gap-4">
          <div class="w-12 h-12 bg-red-500/20 rounded-xl flex items-center justify-center">
            <svg class="w-6 h-6 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div>
            <h4 class="text-lg font-semibold text-red-400">{{ bridgeStore.stats.error_count }} Errors Detected</h4>
            <p class="text-sm text-slate-400">Check the logs for more details</p>
          </div>
        </div>
      </div>
    </div>
  </MainLayout>
</template>

<style scoped></style>
