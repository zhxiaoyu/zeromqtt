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

    // MQTT Configs (Multiple Brokers)
    async getMqttConfigs(): Promise<MqttConfig[]> {
        const response = await api.get<MqttConfig[]>('/config/mqtt')
        return response.data
    },

    async getMqttConfig(id: number): Promise<MqttConfig> {
        const response = await api.get<MqttConfig>(`/config/mqtt/${id}`)
        return response.data
    },

    async addMqttConfig(config: Omit<MqttConfig, 'id'>): Promise<MqttConfig> {
        const response = await api.post<MqttConfig>('/config/mqtt', config)
        return response.data
    },

    async updateMqttConfig(id: number, config: Omit<MqttConfig, 'id'>): Promise<MqttConfig> {
        const response = await api.put<MqttConfig>(`/config/mqtt/${id}`, config)
        return response.data
    },

    async deleteMqttConfig(id: number): Promise<void> {
        await api.delete(`/config/mqtt/${id}`)
    },

    // ZMQ Configs (XPUB/XSUB)
    async getZmqConfigs(): Promise<ZmqConfig[]> {
        const response = await api.get<ZmqConfig[]>('/config/zmq')
        return response.data
    },

    async getZmqConfig(id: number): Promise<ZmqConfig> {
        const response = await api.get<ZmqConfig>(`/config/zmq/${id}`)
        return response.data
    },

    async addZmqConfig(config: Omit<ZmqConfig, 'id'>): Promise<ZmqConfig> {
        const response = await api.post<ZmqConfig>('/config/zmq', config)
        return response.data
    },

    async updateZmqConfig(id: number, config: Omit<ZmqConfig, 'id'>): Promise<ZmqConfig> {
        const response = await api.put<ZmqConfig>(`/config/zmq/${id}`, config)
        return response.data
    },

    async deleteZmqConfig(id: number): Promise<void> {
        await api.delete(`/config/zmq/${id}`)
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
