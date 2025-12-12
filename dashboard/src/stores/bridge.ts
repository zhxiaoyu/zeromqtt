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

// Updated for multi-broker support
export interface MqttConfig {
    id?: number
    name: string
    enabled: boolean
    broker_url: string
    port: number
    client_id: string
    username?: string
    password?: string
    use_tls: boolean
    keep_alive_seconds: number
    clean_session: boolean
}

// Updated for XPUB/XSUB pattern
export type ZmqSocketType = 'xpub' | 'xsub' | 'pub' | 'sub'

export interface ZmqConfig {
    id?: number
    name: string
    enabled: boolean
    socket_type: ZmqSocketType
    bind_endpoint?: string
    connect_endpoints: string[]
    high_water_mark: number
    reconnect_interval_ms: number
}

// Updated with endpoint references
export type EndpointType = 'mqtt' | 'zmq'
export type MappingDirection = 'mqtt_to_zmq' | 'zmq_to_mqtt' | 'mqtt_to_mqtt' | 'zmq_to_zmq' | 'bidirectional'

export interface TopicMapping {
    id: number
    source_endpoint_type: EndpointType
    source_endpoint_id: number
    target_endpoint_type: EndpointType
    target_endpoint_id: number
    source_topic: string
    target_topic: string
    direction: MappingDirection
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

    // Multi-config support
    const mqttConfigs = ref<MqttConfig[]>([])
    const zmqConfigs = ref<ZmqConfig[]>([])
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

    // MQTT Configs (Multiple Brokers)
    async function fetchMqttConfigs() {
        try {
            mqttConfigs.value = await bridgeApi.getMqttConfigs()
        } catch (e) {
            console.error('Failed to fetch MQTT configs:', e)
        }
    }

    async function addMqttConfig(config: Omit<MqttConfig, 'id'>) {
        loading.value = true
        try {
            const newConfig = await bridgeApi.addMqttConfig(config)
            mqttConfigs.value.push(newConfig)
            return newConfig
        } finally {
            loading.value = false
        }
    }

    async function updateMqttConfig(id: number, config: Omit<MqttConfig, 'id'>) {
        loading.value = true
        try {
            const updated = await bridgeApi.updateMqttConfig(id, config)
            const index = mqttConfigs.value.findIndex(c => c.id === id)
            if (index !== -1) {
                mqttConfigs.value[index] = updated
            }
            return updated
        } finally {
            loading.value = false
        }
    }

    async function deleteMqttConfig(id: number) {
        loading.value = true
        try {
            await bridgeApi.deleteMqttConfig(id)
            mqttConfigs.value = mqttConfigs.value.filter(c => c.id !== id)
        } finally {
            loading.value = false
        }
    }

    // ZMQ Configs (XPUB/XSUB)
    async function fetchZmqConfigs() {
        try {
            zmqConfigs.value = await bridgeApi.getZmqConfigs()
        } catch (e) {
            console.error('Failed to fetch ZMQ configs:', e)
        }
    }

    async function addZmqConfig(config: Omit<ZmqConfig, 'id'>) {
        loading.value = true
        try {
            const newConfig = await bridgeApi.addZmqConfig(config)
            zmqConfigs.value.push(newConfig)
            return newConfig
        } finally {
            loading.value = false
        }
    }

    async function updateZmqConfig(id: number, config: Omit<ZmqConfig, 'id'>) {
        loading.value = true
        try {
            const updated = await bridgeApi.updateZmqConfig(id, config)
            const index = zmqConfigs.value.findIndex(c => c.id === id)
            if (index !== -1) {
                zmqConfigs.value[index] = updated
            }
            return updated
        } finally {
            loading.value = false
        }
    }

    async function deleteZmqConfig(id: number) {
        loading.value = true
        try {
            await bridgeApi.deleteZmqConfig(id)
            zmqConfigs.value = zmqConfigs.value.filter(c => c.id !== id)
        } finally {
            loading.value = false
        }
    }

    // Topic Mappings
    async function fetchMappings() {
        try {
            mappings.value = await bridgeApi.getMappings()
        } catch (e) {
            console.error('Failed to fetch mappings:', e)
        }
    }

    async function addMapping(mapping: Omit<TopicMapping, 'id'>) {
        loading.value = true
        try {
            const newMapping = await bridgeApi.addMapping(mapping)
            mappings.value.push(newMapping)
            return newMapping
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
            return updated
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

    return {
        status,
        stats,
        chartData,
        mqttConfigs,
        zmqConfigs,
        mappings,
        loading,
        fetchStatus,
        fetchStats,
        fetchChartData,
        fetchMqttConfigs,
        fetchZmqConfigs,
        fetchMappings,
        addMqttConfig,
        updateMqttConfig,
        deleteMqttConfig,
        addZmqConfig,
        updateZmqConfig,
        deleteZmqConfig,
        addMapping,
        updateMapping,
        deleteMapping
    }
})
