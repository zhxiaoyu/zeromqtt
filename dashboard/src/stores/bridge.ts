import { defineStore } from 'pinia'
import { ref } from 'vue'
import { bridgeApi } from '@/services/bridge'

export interface BridgeStatus {
    state: 'running' | 'stopped' | 'error' | 'connecting'
    uptime_seconds: number
    mqtt_status: 'connected' | 'disconnected' | 'connecting' | 'error'
    zmq_status: 'connected' | 'disconnected' | 'connecting' | 'error'
    version: string
}

export interface MessageStats {
    mqtt_received: number
    mqtt_sent: number
    zmq_received: number
    zmq_sent: number
    messages_per_second: number
    avg_latency_ms: number
    error_count: number
    queue_depth: number
}

export interface MqttConfig {
    id?: number
    broker_url: string
    port: number
    client_id: string
    username?: string
    password?: string
    use_tls: boolean
    keep_alive_seconds: number
    clean_session: boolean
}

export interface ZmqConfig {
    id?: number
    pub_endpoint: string
    sub_endpoint: string
    high_water_mark: number
    reconnect_interval_ms: number
}

export interface TopicMapping {
    id: number
    source_topic: string
    target_topic: string
    direction: 'mqtt_to_zmq' | 'zmq_to_mqtt' | 'bidirectional'
    enabled: boolean
    description?: string
}

export interface ChartData {
    label: string
    data: { timestamp: number; value: number }[]
}

export const useBridgeStore = defineStore('bridge', () => {
    const status = ref<BridgeStatus | null>(null)
    const stats = ref<MessageStats | null>(null)
    const chartData = ref<ChartData[]>([])
    const mqttConfig = ref<MqttConfig | null>(null)
    const zmqConfig = ref<ZmqConfig | null>(null)
    const mappings = ref<TopicMapping[]>([])
    const loading = ref(false)

    async function fetchStatus() {
        try {
            status.value = await bridgeApi.getStatus()
        } catch (e) {
            console.error('Failed to fetch status:', e)
        }
    }

    async function fetchStats() {
        try {
            stats.value = await bridgeApi.getStats()
        } catch (e) {
            console.error('Failed to fetch stats:', e)
        }
    }

    async function fetchChartData() {
        try {
            chartData.value = await bridgeApi.getChartData()
        } catch (e) {
            console.error('Failed to fetch chart data:', e)
        }
    }

    async function fetchMqttConfig() {
        try {
            mqttConfig.value = await bridgeApi.getMqttConfig()
        } catch (e) {
            console.error('Failed to fetch MQTT config:', e)
        }
    }

    async function fetchZmqConfig() {
        try {
            zmqConfig.value = await bridgeApi.getZmqConfig()
        } catch (e) {
            console.error('Failed to fetch ZMQ config:', e)
        }
    }

    async function fetchMappings() {
        try {
            mappings.value = await bridgeApi.getMappings()
        } catch (e) {
            console.error('Failed to fetch mappings:', e)
        }
    }

    async function updateMqttConfig(config: MqttConfig) {
        loading.value = true
        try {
            mqttConfig.value = await bridgeApi.updateMqttConfig(config)
        } finally {
            loading.value = false
        }
    }

    async function updateZmqConfig(config: ZmqConfig) {
        loading.value = true
        try {
            zmqConfig.value = await bridgeApi.updateZmqConfig(config)
        } finally {
            loading.value = false
        }
    }

    async function addMapping(mapping: Omit<TopicMapping, 'id'>) {
        loading.value = true
        try {
            const newMapping = await bridgeApi.addMapping(mapping)
            mappings.value.push(newMapping)
        } finally {
            loading.value = false
        }
    }

    async function deleteMapping(id: number) {
        loading.value = true
        try {
            await bridgeApi.deleteMapping(id)
            mappings.value = mappings.value.filter(m => m.id !== id)
        } finally {
            loading.value = false
        }
    }

    async function updateMapping(id: number, mapping: Omit<TopicMapping, 'id'>) {
        loading.value = true
        try {
            const updated = await bridgeApi.updateMapping(id, mapping)
            const index = mappings.value.findIndex(m => m.id === id)
            if (index !== -1) {
                mappings.value[index] = updated
            }
        } finally {
            loading.value = false
        }
    }

    return {
        status,
        stats,
        chartData,
        mqttConfig,
        zmqConfig,
        mappings,
        loading,
        fetchStatus,
        fetchStats,
        fetchChartData,
        fetchMqttConfig,
        fetchZmqConfig,
        fetchMappings,
        updateMqttConfig,
        updateZmqConfig,
        addMapping,
        deleteMapping,
        updateMapping
    }
})
