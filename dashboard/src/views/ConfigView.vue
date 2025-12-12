<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import Modal from '@/components/Modal.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useBridgeStore, type TopicMapping, type MqttConfig, type ZmqConfig, type MappingDirection, type ZmqSocketType } from '@/stores/bridge'

const bridgeStore = useBridgeStore()

const activeTab = ref<'mqtt' | 'zmq' | 'mappings'>('mqtt')
const showMqttModal = ref(false)
const showZmqModal = ref(false)
const showMappingModal = ref(false)
const showDeleteConfirm = ref(false)
const saving = ref(false)

// Currently editing
const editingMqttId = ref<number | null>(null)
const editingZmqId = ref<number | null>(null)
const editingMappingId = ref<number | null>(null)
const deletingType = ref<'mqtt' | 'zmq' | 'mapping' | null>(null)
const deletingId = ref<number | null>(null)

// Form copies
const mqttForm = ref({
  name: 'Default',
  enabled: true,
  broker_url: 'localhost',
  port: 1883,
  client_id: 'zeromqtt-bridge',
  username: '',
  password: '',
  use_tls: false,
  keep_alive_seconds: 60,
  clean_session: true
})

const zmqForm = ref({
  name: 'Default',
  enabled: true,
  socket_type: 'xpub' as ZmqSocketType,
  bind_endpoint: 'tcp://*:5555',
  connect_endpoints_raw: '',
  high_water_mark: 1000,
  reconnect_interval_ms: 1000
})

const mappingForm = ref({
  source_endpoint_type: 'mqtt' as 'mqtt' | 'zmq',
  source_endpoint_id: 1,
  target_endpoint_type: 'zmq' as 'mqtt' | 'zmq',
  target_endpoint_id: 1,
  source_topic: '',
  target_topic: '',
  direction: 'mqtt_to_zmq' as MappingDirection,
  enabled: true,
  description: ''
})

const resetMqttForm = () => {
  mqttForm.value = {
    name: '',
    enabled: true,
    broker_url: 'localhost',
    port: 1883,
    client_id: 'zeromqtt-bridge',
    username: '',
    password: '',
    use_tls: false,
    keep_alive_seconds: 60,
    clean_session: true
  }
  editingMqttId.value = null
}

const resetZmqForm = () => {
  zmqForm.value = {
    name: '',
    enabled: true,
    socket_type: 'xpub',
    bind_endpoint: 'tcp://*:5555',
    connect_endpoints_raw: '',
    high_water_mark: 1000,
    reconnect_interval_ms: 1000
  }
  editingZmqId.value = null
}

const resetMappingForm = () => {
  mappingForm.value = {
    source_endpoint_type: 'mqtt',
    source_endpoint_id: bridgeStore.mqttConfigs[0]?.id || 1,
    target_endpoint_type: 'zmq',
    target_endpoint_id: bridgeStore.zmqConfigs[0]?.id || 1,
    source_topic: '',
    target_topic: '',
    direction: 'mqtt_to_zmq',
    enabled: true,
    description: ''
  }
  editingMappingId.value = null
}

onMounted(async () => {
  await Promise.all([
    bridgeStore.fetchMqttConfigs(),
    bridgeStore.fetchZmqConfigs(),
    bridgeStore.fetchMappings()
  ])
})

// MQTT CRUD
const openAddMqttModal = () => {
  resetMqttForm()
  showMqttModal.value = true
}

const openEditMqttModal = (config: MqttConfig) => {
  editingMqttId.value = config.id!
  mqttForm.value = {
    name: config.name,
    enabled: config.enabled,
    broker_url: config.broker_url,
    port: config.port,
    client_id: config.client_id,
    username: config.username || '',
    password: config.password || '',
    use_tls: config.use_tls,
    keep_alive_seconds: config.keep_alive_seconds,
    clean_session: config.clean_session
  }
  showMqttModal.value = true
}

const saveMqttConfig = async () => {
  saving.value = true
  try {
    if (editingMqttId.value !== null) {
      await bridgeStore.updateMqttConfig(editingMqttId.value, mqttForm.value)
    } else {
      await bridgeStore.addMqttConfig(mqttForm.value)
    }
    showMqttModal.value = false
    resetMqttForm()
  } finally {
    saving.value = false
  }
}

// ZMQ CRUD
const openAddZmqModal = () => {
  resetZmqForm()
  showZmqModal.value = true
}

const openEditZmqModal = (config: ZmqConfig) => {
  editingZmqId.value = config.id!
  zmqForm.value = {
    name: config.name,
    enabled: config.enabled,
    socket_type: config.socket_type,
    bind_endpoint: config.bind_endpoint || '',
    connect_endpoints_raw: config.connect_endpoints.join(', '),
    high_water_mark: config.high_water_mark,
    reconnect_interval_ms: config.reconnect_interval_ms
  }
  showZmqModal.value = true
}

const saveZmqConfig = async () => {
  saving.value = true
  try {
    const payload = {
      name: zmqForm.value.name,
      enabled: zmqForm.value.enabled,
      socket_type: zmqForm.value.socket_type,
      bind_endpoint: zmqForm.value.bind_endpoint || undefined,
      connect_endpoints: zmqForm.value.connect_endpoints_raw.split(',').map(s => s.trim()).filter(s => s),
      high_water_mark: zmqForm.value.high_water_mark,
      reconnect_interval_ms: zmqForm.value.reconnect_interval_ms
    }
    if (editingZmqId.value !== null) {
      await bridgeStore.updateZmqConfig(editingZmqId.value, payload)
    } else {
      await bridgeStore.addZmqConfig(payload)
    }
    showZmqModal.value = false
    resetZmqForm()
  } finally {
    saving.value = false
  }
}

// Mapping CRUD
const openAddMappingModal = () => {
  resetMappingForm()
  showMappingModal.value = true
}

const openEditMappingModal = (mapping: TopicMapping) => {
  editingMappingId.value = mapping.id
  mappingForm.value = {
    source_endpoint_type: mapping.source_endpoint_type,
    source_endpoint_id: mapping.source_endpoint_id,
    target_endpoint_type: mapping.target_endpoint_type,
    target_endpoint_id: mapping.target_endpoint_id,
    source_topic: mapping.source_topic,
    target_topic: mapping.target_topic,
    direction: mapping.direction,
    enabled: mapping.enabled,
    description: mapping.description || ''
  }
  showMappingModal.value = true
}

const saveMapping = async () => {
  saving.value = true
  try {
    if (editingMappingId.value !== null) {
      await bridgeStore.updateMapping(editingMappingId.value, mappingForm.value)
    } else {
      await bridgeStore.addMapping(mappingForm.value)
    }
    showMappingModal.value = false
    resetMappingForm()
  } finally {
    saving.value = false
  }
}

// Delete operations
const confirmDelete = (type: 'mqtt' | 'zmq' | 'mapping', id: number) => {
  deletingType.value = type
  deletingId.value = id
  showDeleteConfirm.value = true
}

const executeDelete = async () => {
  if (deletingId.value !== null && deletingType.value) {
    if (deletingType.value === 'mqtt') {
      await bridgeStore.deleteMqttConfig(deletingId.value)
    } else if (deletingType.value === 'zmq') {
      await bridgeStore.deleteZmqConfig(deletingId.value)
    } else {
      await bridgeStore.deleteMapping(deletingId.value)
    }
  }
  showDeleteConfirm.value = false
  deletingType.value = null
  deletingId.value = null
}

// Helper functions
const getSocketTypeLabel = (type: ZmqSocketType) => {
  switch (type) {
    case 'xpub': return 'XPUB (Proxy)'
    case 'xsub': return 'XSUB (Proxy)'
    case 'pub': return 'PUB'
    case 'sub': return 'SUB'
    default: return type
  }
}

const getDirectionLabel = (direction: MappingDirection) => {
  switch (direction) {
    case 'mqtt_to_zmq': return 'MQTT → ZMQ'
    case 'zmq_to_mqtt': return 'ZMQ → MQTT'
    case 'mqtt_to_mqtt': return 'MQTT → MQTT'
    case 'zmq_to_zmq': return 'ZMQ → ZMQ'
    case 'bidirectional': return 'Bidirectional'
    default: return direction
  }
}

const getDirectionColor = (direction: MappingDirection) => {
  switch (direction) {
    case 'mqtt_to_zmq': return 'text-cyan-400 bg-cyan-500/20'
    case 'zmq_to_mqtt': return 'text-purple-400 bg-purple-500/20'
    case 'mqtt_to_mqtt': return 'text-blue-400 bg-blue-500/20'
    case 'zmq_to_zmq': return 'text-green-400 bg-green-500/20'
    case 'bidirectional': return 'text-amber-400 bg-amber-500/20'
    default: return 'text-slate-400 bg-slate-500/20'
  }
}

const getEndpointName = (type: 'mqtt' | 'zmq', id: number) => {
  if (type === 'mqtt') {
    const config = bridgeStore.mqttConfigs.find(c => c.id === id)
    return config?.name || `MQTT #${id}`
  } else {
    const config = bridgeStore.zmqConfigs.find(c => c.id === id)
    return config?.name || `ZMQ #${id}`
  }
}

// Computed options for dropdowns
const sourceEndpoints = computed(() => {
  if (mappingForm.value.source_endpoint_type === 'mqtt') {
    return bridgeStore.mqttConfigs.map(c => ({ id: c.id!, name: c.name }))
  } else {
    return bridgeStore.zmqConfigs.map(c => ({ id: c.id!, name: c.name }))
  }
})

const targetEndpoints = computed(() => {
  if (mappingForm.value.target_endpoint_type === 'mqtt') {
    return bridgeStore.mqttConfigs.map(c => ({ id: c.id!, name: c.name }))
  } else {
    return bridgeStore.zmqConfigs.map(c => ({ id: c.id!, name: c.name }))
  }
})
</script>

<template>
  <MainLayout>
    <template #title>Configuration</template>
    
    <div class="space-y-6">
      <!-- Tabs -->
      <div class="flex gap-2">
        <button
          v-for="tab in [{ key: 'mqtt', label: 'MQTT Brokers' }, { key: 'zmq', label: 'ZeroMQ' }, { key: 'mappings', label: 'Topic Mappings' }]"
          :key="tab.key"
          @click="activeTab = tab.key as any"
          class="px-4 py-2 rounded-lg font-medium transition-smooth"
          :class="activeTab === tab.key 
            ? 'bg-gradient-to-r from-cyan-500/20 to-blue-500/20 text-cyan-400 border border-cyan-500/30' 
            : 'text-slate-400 hover:text-white hover:bg-slate-800'"
        >
          {{ tab.label }}
          <span v-if="tab.key === 'mqtt'" class="ml-1 text-xs opacity-70">({{ bridgeStore.mqttConfigs.length }})</span>
          <span v-if="tab.key === 'zmq'" class="ml-1 text-xs opacity-70">({{ bridgeStore.zmqConfigs.length }})</span>
        </button>
      </div>

      <!-- MQTT Brokers List -->
      <div v-if="activeTab === 'mqtt'" class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-white">MQTT Broker Connections</h3>
          <button @click="openAddMqttModal" class="btn-primary flex items-center gap-2">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Broker
          </button>
        </div>

        <div class="grid gap-4">
          <div v-for="config in bridgeStore.mqttConfigs" :key="config.id" class="glass-card p-4">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-3 h-3 rounded-full" :class="config.enabled ? 'bg-emerald-500' : 'bg-slate-500'"></div>
                <div>
                  <h4 class="font-semibold text-white">{{ config.name }}</h4>
                  <p class="text-sm text-slate-400">{{ config.broker_url }}:{{ config.port }}</p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-1 rounded" :class="config.use_tls ? 'bg-green-500/20 text-green-400' : 'bg-slate-500/20 text-slate-400'">
                  {{ config.use_tls ? 'TLS' : 'Plain' }}
                </span>
                <button @click="openEditMqttModal(config)" class="text-cyan-400 hover:text-cyan-300 p-1">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button @click="confirmDelete('mqtt', config.id!)" class="text-red-400 hover:text-red-300 p-1">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div v-if="!bridgeStore.mqttConfigs.length" class="glass-card p-8 text-center text-slate-400">
            No MQTT brokers configured. Click "Add Broker" to create one.
          </div>
        </div>
      </div>

      <!-- ZeroMQ Configs List -->
      <div v-if="activeTab === 'zmq'" class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-white">ZeroMQ Endpoints</h3>
          <button @click="openAddZmqModal" class="btn-primary flex items-center gap-2">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Endpoint
          </button>
        </div>

        <div class="grid gap-4">
          <div v-for="config in bridgeStore.zmqConfigs" :key="config.id" class="glass-card p-4">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <div class="w-3 h-3 rounded-full" :class="config.enabled ? 'bg-emerald-500' : 'bg-slate-500'"></div>
                <div>
                  <h4 class="font-semibold text-white">{{ config.name }}</h4>
                  <p class="text-sm text-slate-400">
                    <span v-if="config.bind_endpoint">Bind: {{ config.bind_endpoint }}</span>
                    <span v-if="config.connect_endpoints?.length"> | Connect: {{ config.connect_endpoints.join(', ') }}</span>
                  </p>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-1 rounded bg-purple-500/20 text-purple-400 uppercase">
                  {{ config.socket_type }}
                </span>
                <button @click="openEditZmqModal(config)" class="text-cyan-400 hover:text-cyan-300 p-1">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button @click="confirmDelete('zmq', config.id!)" class="text-red-400 hover:text-red-300 p-1">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div v-if="!bridgeStore.zmqConfigs.length" class="glass-card p-8 text-center text-slate-400">
            No ZeroMQ endpoints configured. Click "Add Endpoint" to create one.
          </div>
        </div>
      </div>

      <!-- Topic Mappings -->
      <div v-if="activeTab === 'mappings'" class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-white">Topic Mappings</h3>
          <button @click="openAddMappingModal" class="btn-primary flex items-center gap-2">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Mapping
          </button>
        </div>

        <div class="glass-card overflow-hidden">
          <table class="w-full">
            <thead class="bg-slate-800/50">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase">Source</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase">Target</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase">Direction</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase">Status</th>
                <th class="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-700/50">
              <tr v-for="mapping in bridgeStore.mappings" :key="mapping.id" class="hover:bg-slate-800/30">
                <td class="px-4 py-3">
                  <div class="text-xs text-slate-500 mb-1">{{ getEndpointName(mapping.source_endpoint_type, mapping.source_endpoint_id) }}</div>
                  <code class="text-cyan-400 bg-cyan-500/10 px-2 py-0.5 rounded text-sm">{{ mapping.source_topic }}</code>
                </td>
                <td class="px-4 py-3">
                  <div class="text-xs text-slate-500 mb-1">{{ getEndpointName(mapping.target_endpoint_type, mapping.target_endpoint_id) }}</div>
                  <code class="text-purple-400 bg-purple-500/10 px-2 py-0.5 rounded text-sm">{{ mapping.target_topic }}</code>
                </td>
                <td class="px-4 py-3">
                  <span class="px-2 py-1 rounded text-xs font-medium" :class="getDirectionColor(mapping.direction)">
                    {{ getDirectionLabel(mapping.direction) }}
                  </span>
                </td>
                <td class="px-4 py-3">
                  <span class="px-2 py-1 rounded text-xs font-medium" :class="mapping.enabled ? 'text-emerald-400 bg-emerald-500/20' : 'text-slate-400 bg-slate-500/20'">
                    {{ mapping.enabled ? 'Enabled' : 'Disabled' }}
                  </span>
                </td>
                <td class="px-4 py-3">
                  <div class="flex items-center gap-2">
                    <button @click="openEditMappingModal(mapping)" class="text-cyan-400 hover:text-cyan-300 p-1">
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button @click="confirmDelete('mapping', mapping.id)" class="text-red-400 hover:text-red-300 p-1">
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </td>
              </tr>
              <tr v-if="!bridgeStore.mappings.length">
                <td colspan="5" class="px-6 py-8 text-center text-slate-400">
                  No topic mappings configured. Click "Add Mapping" to create one.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- MQTT Modal -->
    <Modal :show="showMqttModal" :title="editingMqttId ? 'Edit MQTT Broker' : 'Add MQTT Broker'" size="lg" @close="showMqttModal = false">
      <form @submit.prevent="saveMqttConfig" class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div class="col-span-2">
            <label class="block text-sm font-medium text-slate-300 mb-2">Name</label>
            <input v-model="mqttForm.name" type="text" class="input-dark w-full" placeholder="Primary Broker" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Broker URL</label>
            <input v-model="mqttForm.broker_url" type="text" class="input-dark w-full" placeholder="localhost" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Port</label>
            <input v-model.number="mqttForm.port" type="number" class="input-dark w-full" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Client ID</label>
            <input v-model="mqttForm.client_id" type="text" class="input-dark w-full" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Keep Alive (s)</label>
            <input v-model.number="mqttForm.keep_alive_seconds" type="number" class="input-dark w-full" />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Username</label>
            <input v-model="mqttForm.username" type="text" class="input-dark w-full" placeholder="Optional" />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Password</label>
            <input v-model="mqttForm.password" type="password" class="input-dark w-full" placeholder="Optional" />
          </div>
        </div>
        <div class="flex gap-6">
          <label class="flex items-center gap-2 cursor-pointer">
            <input v-model="mqttForm.enabled" type="checkbox" class="w-4 h-4 rounded bg-slate-700 text-cyan-500" />
            <span class="text-slate-300">Enabled</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input v-model="mqttForm.use_tls" type="checkbox" class="w-4 h-4 rounded bg-slate-700 text-cyan-500" />
            <span class="text-slate-300">Use TLS</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input v-model="mqttForm.clean_session" type="checkbox" class="w-4 h-4 rounded bg-slate-700 text-cyan-500" />
            <span class="text-slate-300">Clean Session</span>
          </label>
        </div>
      </form>
      <template #footer>
        <button @click="showMqttModal = false" class="btn-secondary">Cancel</button>
        <button @click="saveMqttConfig" :disabled="saving" class="btn-primary">
          {{ saving ? 'Saving...' : (editingMqttId ? 'Update' : 'Add') }}
        </button>
      </template>
    </Modal>

    <!-- ZMQ Modal -->
    <Modal :show="showZmqModal" :title="editingZmqId ? 'Edit ZMQ Endpoint' : 'Add ZMQ Endpoint'" size="lg" @close="showZmqModal = false">
      <form @submit.prevent="saveZmqConfig" class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Name</label>
            <input v-model="zmqForm.name" type="text" class="input-dark w-full" placeholder="XPUB Proxy" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Socket Type</label>
            <select v-model="zmqForm.socket_type" class="input-dark w-full">
              <option value="xpub">XPUB (Proxy - serves subscribers)</option>
              <option value="xsub">XSUB (Proxy - receives from publishers)</option>
              <option value="pub">PUB (Standard publisher)</option>
              <option value="sub">SUB (Standard subscriber)</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Bind Endpoint</label>
            <input v-model="zmqForm.bind_endpoint" type="text" class="input-dark w-full" placeholder="tcp://*:5555" />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Connect Endpoints (comma-separated)</label>
            <input v-model="zmqForm.connect_endpoints_raw" type="text" class="input-dark w-full" placeholder="tcp://host:5555, tcp://host:5556" />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">High Water Mark</label>
            <input v-model.number="zmqForm.high_water_mark" type="number" class="input-dark w-full" />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Reconnect Interval (ms)</label>
            <input v-model.number="zmqForm.reconnect_interval_ms" type="number" class="input-dark w-full" />
          </div>
        </div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input v-model="zmqForm.enabled" type="checkbox" class="w-4 h-4 rounded bg-slate-700 text-cyan-500" />
          <span class="text-slate-300">Enabled</span>
        </label>
      </form>
      <template #footer>
        <button @click="showZmqModal = false" class="btn-secondary">Cancel</button>
        <button @click="saveZmqConfig" :disabled="saving" class="btn-primary">
          {{ saving ? 'Saving...' : (editingZmqId ? 'Update' : 'Add') }}
        </button>
      </template>
    </Modal>

    <!-- Mapping Modal -->
    <Modal :show="showMappingModal" :title="editingMappingId ? 'Edit Topic Mapping' : 'Add Topic Mapping'" size="lg" @close="showMappingModal = false">
      <form @submit.prevent="saveMapping" class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Source Type</label>
            <select v-model="mappingForm.source_endpoint_type" class="input-dark w-full">
              <option value="mqtt">MQTT</option>
              <option value="zmq">ZeroMQ</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Source Endpoint</label>
            <select v-model.number="mappingForm.source_endpoint_id" class="input-dark w-full">
              <option v-for="ep in sourceEndpoints" :key="ep.id" :value="ep.id">{{ ep.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Target Type</label>
            <select v-model="mappingForm.target_endpoint_type" class="input-dark w-full">
              <option value="mqtt">MQTT</option>
              <option value="zmq">ZeroMQ</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Target Endpoint</label>
            <select v-model.number="mappingForm.target_endpoint_id" class="input-dark w-full">
              <option v-for="ep in targetEndpoints" :key="ep.id" :value="ep.id">{{ ep.name }}</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Source Topic</label>
            <input v-model="mappingForm.source_topic" type="text" class="input-dark w-full" placeholder="sensors/#" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Target Topic</label>
            <input v-model="mappingForm.target_topic" type="text" class="input-dark w-full" placeholder="zmq.sensors" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Direction</label>
            <select v-model="mappingForm.direction" class="input-dark w-full">
              <option value="mqtt_to_zmq">MQTT → ZMQ</option>
              <option value="zmq_to_mqtt">ZMQ → MQTT</option>
              <option value="mqtt_to_mqtt">MQTT → MQTT</option>
              <option value="zmq_to_zmq">ZMQ → ZMQ</option>
              <option value="bidirectional">Bidirectional</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Description</label>
            <input v-model="mappingForm.description" type="text" class="input-dark w-full" placeholder="Optional" />
          </div>
        </div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input v-model="mappingForm.enabled" type="checkbox" class="w-4 h-4 rounded bg-slate-700 text-cyan-500" />
          <span class="text-slate-300">Enabled</span>
        </label>
      </form>
      <template #footer>
        <button @click="showMappingModal = false" class="btn-secondary">Cancel</button>
        <button @click="saveMapping" :disabled="saving" class="btn-primary">
          {{ saving ? 'Saving...' : (editingMappingId ? 'Update' : 'Add') }}
        </button>
      </template>
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      title="Confirm Delete"
      :message="`Are you sure you want to delete this ${deletingType}? This action cannot be undone.`"
      type="danger"
      confirm-text="Delete"
      @confirm="executeDelete"
      @cancel="showDeleteConfirm = false"
    />
  </MainLayout>
</template>

<style scoped></style>
