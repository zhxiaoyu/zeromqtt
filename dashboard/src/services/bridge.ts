import api from './api'
import type { BridgeStatus, MessageStats, ChartData, MqttConfig, ZmqConfig, TopicMapping } from '@/stores/bridge'

export const bridgeApi = {
    // Status endpoints
    async getStatus(): Promise<BridgeStatus> {
        const response = await api.get<BridgeStatus>('/status')
        return response.data
    },

    async getStats(): Promise<MessageStats> {
        const response = await api.get<MessageStats>('/status/stats')
        return response.data
    },

    async getChartData(): Promise<ChartData[]> {
        const response = await api.get<ChartData[]>('/status/chart')
        return response.data
    },

    // MQTT Config
    async getMqttConfig(): Promise<MqttConfig> {
        const response = await api.get<MqttConfig>('/config/mqtt')
        return response.data
    },

    async updateMqttConfig(config: MqttConfig): Promise<MqttConfig> {
        const response = await api.put<MqttConfig>('/config/mqtt', config)
        return response.data
    },

    // ZMQ Config
    async getZmqConfig(): Promise<ZmqConfig> {
        const response = await api.get<ZmqConfig>('/config/zmq')
        return response.data
    },

    async updateZmqConfig(config: ZmqConfig): Promise<ZmqConfig> {
        const response = await api.put<ZmqConfig>('/config/zmq', config)
        return response.data
    },

    // Topic Mappings
    async getMappings(): Promise<TopicMapping[]> {
        const response = await api.get<TopicMapping[]>('/config/mappings')
        return response.data
    },

    async addMapping(mapping: Omit<TopicMapping, 'id'>): Promise<TopicMapping> {
        const response = await api.post<TopicMapping>('/config/mappings', mapping)
        return response.data
    },

    async updateMapping(id: number, mapping: Omit<TopicMapping, 'id'>): Promise<TopicMapping> {
        const response = await api.put<TopicMapping>(`/config/mappings/${id}`, mapping)
        return response.data
    },

    async deleteMapping(id: number): Promise<void> {
        await api.delete(`/config/mappings/${id}`)
    }
}
