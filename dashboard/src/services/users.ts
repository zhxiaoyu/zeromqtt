import api from './api'

// User types
export interface User {
    id: number
    username: string
    is_default: boolean
    created_at: number
    updated_at: number
}

export interface CreateUserRequest {
    username: string
    password: string
}

export interface UpdateUserRequest {
    username: string
}

export interface ChangePasswordRequest {
    current_password?: string
    new_password: string
}

// User API functions
export const usersApi = {
    // Get all users
    async getUsers(): Promise<User[]> {
        const response = await api.get('/users')
        return response.data
    },

    // Get user by ID
    async getUser(id: number): Promise<User> {
        const response = await api.get(`/users/${id}`)
        return response.data
    },

    // Create a new user
    async createUser(data: CreateUserRequest): Promise<User> {
        const response = await api.post('/users', data)
        return response.data
    },

    // Update user
    async updateUser(id: number, data: UpdateUserRequest): Promise<User> {
        const response = await api.put(`/users/${id}`, data)
        return response.data
    },

    // Change user password
    async changePassword(id: number, data: ChangePasswordRequest): Promise<void> {
        await api.post(`/users/${id}/password`, data)
    },

    // Delete user
    async deleteUser(id: number): Promise<void> {
        await api.delete(`/users/${id}`)
    }
}

export default usersApi
