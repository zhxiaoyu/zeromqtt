import api from './api'

export interface LoginResponse {
    token: string
    token_type: string
    expires_in: number
}

export interface MeResponse {
    username: string
}

export const authApi = {
    async login(username: string, password: string): Promise<LoginResponse> {
        const response = await api.post<LoginResponse>('/auth/login', {
            username,
            password
        })
        return response.data
    },

    async me(): Promise<MeResponse> {
        const response = await api.get<MeResponse>('/auth/me')
        return response.data
    }
}
