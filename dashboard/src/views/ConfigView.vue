<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import Modal from '@/components/Modal.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { useBridgeStore, type TopicMapping } from '@/stores/bridge'

const bridgeStore = useBridgeStore()

const activeTab = ref<'mqtt' | 'zmq' | 'mappings'>('mqtt')
const showMappingModal = ref(false)
const showDeleteConfirm = ref(false)
const savingMqtt = ref(false)
const savingZmq = ref(false)
const savingMapping = ref(false)

// Currently editing mapping (null for new)
const editingMappingId = ref<number | null>(null)
const deletingMappingId = ref<number | null>(null)

// Local form copies
const mqttForm = ref({
  broker_url: '',
  port: 1883,
  client_id: '',
  username: '',
  password: '',
  use_tls: false,
  keep_alive_seconds: 60,
  clean_session: true
})

const zmqForm = ref({
  pub_endpoint: '',
  sub_endpoint: '',
  high_water_mark: 1000,
  reconnect_interval_ms: 1000
})

const mappingForm = ref({
  source_topic: '',
  target_topic: '',
  direction: 'mqtt_to_zmq' as 'mqtt_to_zmq' | 'zmq_to_mqtt' | 'bidirectional',
  enabled: true,
  description: ''
})

const resetMappingForm = () => {
  mappingForm.value = {
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
    bridgeStore.fetchMqttConfig(),
    bridgeStore.fetchZmqConfig(),
    bridgeStore.fetchMappings()
  ])
  
  // Initialize forms with fetched data
  if (bridgeStore.mqttConfig) {
    mqttForm.value = {
      broker_url: bridgeStore.mqttConfig.broker_url,
      port: bridgeStore.mqttConfig.port,
      client_id: bridgeStore.mqttConfig.client_id,
      username: bridgeStore.mqttConfig.username ?? '',
      password: bridgeStore.mqttConfig.password ?? '',
      use_tls: bridgeStore.mqttConfig.use_tls,
      keep_alive_seconds: bridgeStore.mqttConfig.keep_alive_seconds,
      clean_session: bridgeStore.mqttConfig.clean_session
    }
  }
  if (bridgeStore.zmqConfig) {
    zmqForm.value = { ...bridgeStore.zmqConfig }
  }
})

const saveMqttConfig = async () => {
  savingMqtt.value = true
  try {
    await bridgeStore.updateMqttConfig(mqttForm.value)
  } finally {
    savingMqtt.value = false
  }
}

const saveZmqConfig = async () => {
  savingZmq.value = true
  try {
    await bridgeStore.updateZmqConfig(zmqForm.value)
  } finally {
    savingZmq.value = false
  }
}

const openAddMappingModal = () => {
  resetMappingForm()
  showMappingModal.value = true
}

const openEditMappingModal = (mapping: TopicMapping) => {
  editingMappingId.value = mapping.id
  mappingForm.value = {
    source_topic: mapping.source_topic,
    target_topic: mapping.target_topic,
    direction: mapping.direction,
    enabled: mapping.enabled,
    description: mapping.description ?? ''
  }
  showMappingModal.value = true
}

const closeMappingModal = () => {
  showMappingModal.value = false
  resetMappingForm()
}

const saveMapping = async () => {
  savingMapping.value = true
  try {
    if (editingMappingId.value !== null) {
      await bridgeStore.updateMapping(editingMappingId.value, mappingForm.value)
    } else {
      await bridgeStore.addMapping(mappingForm.value)
    }
    closeMappingModal()
  } finally {
    savingMapping.value = false
  }
}

const confirmDeleteMapping = (id: number) => {
  deletingMappingId.value = id
  showDeleteConfirm.value = true
}

const deleteMapping = async () => {
  if (deletingMappingId.value !== null) {
    await bridgeStore.deleteMapping(deletingMappingId.value)
    deletingMappingId.value = null
    showDeleteConfirm.value = false
  }
}

const getDirectionLabel = (direction: string) => {
  switch (direction) {
    case 'mqtt_to_zmq': return 'MQTT → ZMQ'
    case 'zmq_to_mqtt': return 'ZMQ → MQTT'
    case 'bidirectional': return 'Bidirectional'
    default: return direction
  }
}

const getDirectionColor = (direction: string) => {
  switch (direction) {
    case 'mqtt_to_zmq': return 'text-cyan-400 bg-cyan-500/20'
    case 'zmq_to_mqtt': return 'text-purple-400 bg-purple-500/20'
    case 'bidirectional': return 'text-amber-400 bg-amber-500/20'
    default: return 'text-slate-400 bg-slate-500/20'
  }
}
</script>

<template>
  <MainLayout>
    <template #title>Configuration</template>
    
    <div class="space-y-6">
      <!-- Tabs -->
      <div class="flex gap-2">
        <button
          v-for="tab in ['mqtt', 'zmq', 'mappings'] as const"
          :key="tab"
          @click="activeTab = tab"
          class="px-4 py-2 rounded-lg font-medium transition-smooth"
          :class="activeTab === tab 
            ? 'bg-gradient-to-r from-cyan-500/20 to-blue-500/20 text-cyan-400 border border-cyan-500/30' 
            : 'text-slate-400 hover:text-white hover:bg-slate-800'"
        >
          {{ tab === 'mqtt' ? 'MQTT' : tab === 'zmq' ? 'ZeroMQ' : 'Topic Mappings' }}
        </button>
      </div>

      <!-- MQTT Config -->
      <div v-if="activeTab === 'mqtt'" class="glass-card p-6">
        <h3 class="text-lg font-semibold text-white mb-6">MQTT Connection Settings</h3>
        
        <form @submit.prevent="saveMqttConfig" class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Broker URL</label>
              <input v-model="mqttForm.broker_url" type="text" class="input-dark w-full" placeholder="localhost" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Port</label>
              <input v-model.number="mqttForm.port" type="number" class="input-dark w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Client ID</label>
              <input v-model="mqttForm.client_id" type="text" class="input-dark w-full" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Keep Alive (seconds)</label>
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
              <input v-model="mqttForm.use_tls" type="checkbox" class="w-4 h-4 rounded bg-slate-700 border-slate-600 text-cyan-500 focus:ring-cyan-500" />
              <span class="text-slate-300">Use TLS</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input v-model="mqttForm.clean_session" type="checkbox" class="w-4 h-4 rounded bg-slate-700 border-slate-600 text-cyan-500 focus:ring-cyan-500" />
              <span class="text-slate-300">Clean Session</span>
            </label>
          </div>

          <div class="flex justify-end">
            <button type="submit" :disabled="savingMqtt" class="btn-primary">
              {{ savingMqtt ? 'Saving...' : 'Save Changes' }}
            </button>
          </div>
        </form>
      </div>

      <!-- ZeroMQ Config -->
      <div v-if="activeTab === 'zmq'" class="glass-card p-6">
        <h3 class="text-lg font-semibold text-white mb-6">ZeroMQ Connection Settings</h3>
        
        <form @submit.prevent="saveZmqConfig" class="space-y-6">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Publisher Endpoint</label>
              <input v-model="zmqForm.pub_endpoint" type="text" class="input-dark w-full" placeholder="tcp://*:5555" />
            </div>
            <div>
              <label class="block text-sm font-medium text-slate-300 mb-2">Subscriber Endpoint</label>
              <input v-model="zmqForm.sub_endpoint" type="text" class="input-dark w-full" placeholder="tcp://*:5556" />
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

          <div class="flex justify-end">
            <button type="submit" :disabled="savingZmq" class="btn-primary">
              {{ savingZmq ? 'Saving...' : 'Save Changes' }}
            </button>
          </div>
        </form>
      </div>

      <!-- Topic Mappings -->
      <div v-if="activeTab === 'mappings'" class="space-y-6">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-white">Topic Mappings</h3>
          <button @click="openAddMappingModal" class="btn-primary flex items-center gap-2">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Add Mapping
          </button>
        </div>

        <!-- Mappings table -->
        <div class="glass-card overflow-hidden">
          <table class="w-full">
            <thead class="bg-slate-800/50">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Source</th>
                <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Target</th>
                <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Direction</th>
                <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Status</th>
                <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">Actions</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-700/50">
              <tr v-for="mapping in bridgeStore.mappings" :key="mapping.id" class="hover:bg-slate-800/30 transition-colors">
                <td class="px-6 py-4">
                  <code class="text-cyan-400 bg-cyan-500/10 px-2 py-1 rounded">{{ mapping.source_topic }}</code>
                </td>
                <td class="px-6 py-4">
                  <code class="text-purple-400 bg-purple-500/10 px-2 py-1 rounded">{{ mapping.target_topic }}</code>
                </td>
                <td class="px-6 py-4">
                  <span class="px-2 py-1 rounded text-xs font-medium" :class="getDirectionColor(mapping.direction)">
                    {{ getDirectionLabel(mapping.direction) }}
                  </span>
                </td>
                <td class="px-6 py-4">
                  <span 
                    class="px-2 py-1 rounded text-xs font-medium"
                    :class="mapping.enabled ? 'text-emerald-400 bg-emerald-500/20' : 'text-slate-400 bg-slate-500/20'"
                  >
                    {{ mapping.enabled ? 'Enabled' : 'Disabled' }}
                  </span>
                </td>
                <td class="px-6 py-4">
                  <div class="flex items-center gap-2">
                    <!-- Edit button -->
                    <button 
                      @click="openEditMappingModal(mapping)"
                      class="text-cyan-400 hover:text-cyan-300 transition-colors p-1"
                      title="Edit mapping"
                    >
                      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <!-- Delete button -->
                    <button 
                      @click="confirmDeleteMapping(mapping.id)"
                      class="text-red-400 hover:text-red-300 transition-colors p-1"
                      title="Delete mapping"
                    >
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

    <!-- Add/Edit Mapping Modal -->
    <Modal 
      :show="showMappingModal" 
      :title="editingMappingId !== null ? 'Edit Topic Mapping' : 'New Topic Mapping'"
      size="lg"
      @close="closeMappingModal"
    >
      <form @submit.prevent="saveMapping" class="space-y-4">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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
              <option value="mqtt_to_zmq">MQTT → ZeroMQ</option>
              <option value="zmq_to_mqtt">ZeroMQ → MQTT</option>
              <option value="bidirectional">Bidirectional</option>
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-300 mb-2">Description</label>
            <input v-model="mappingForm.description" type="text" class="input-dark w-full" placeholder="Optional description" />
          </div>
        </div>
        <div class="flex items-center gap-2">
          <input v-model="mappingForm.enabled" type="checkbox" class="w-4 h-4 rounded bg-slate-700 border-slate-600 text-cyan-500" id="mapping-enabled" />
          <label for="mapping-enabled" class="text-slate-300 cursor-pointer">Enabled</label>
        </div>
      </form>

      <template #footer>
        <button @click="closeMappingModal" class="btn-secondary">Cancel</button>
        <button @click="saveMapping" :disabled="savingMapping" class="btn-primary">
          {{ savingMapping ? 'Saving...' : (editingMappingId !== null ? 'Update Mapping' : 'Add Mapping') }}
        </button>
      </template>
    </Modal>

    <!-- Delete Confirmation Dialog -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      title="Delete Mapping"
      message="Are you sure you want to delete this topic mapping? This action cannot be undone."
      type="danger"
      confirm-text="Delete"
      @confirm="deleteMapping"
      @cancel="showDeleteConfirm = false"
    />
  </MainLayout>
</template>

<style scoped></style>
