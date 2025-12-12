import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { authApi } from '@/services/auth'

export interface User {
    username: string
}

export const useAuthStore = defineStore('auth', () => {
    const token = ref<string | null>(localStorage.getItem('token'))
    const user = ref<User | null>(null)
    const loading = ref(false)
    const error = ref<string | null>(null)

    const isAuthenticated = computed(() => !!token.value)

    async function login(username: string, password: string) {
        loading.value = true
        error.value = null

        try {
            const response = await authApi.login(username, password)
            token.value = response.token
            localStorage.setItem('token', response.token)

            // Fetch user info
            const userInfo = await authApi.me()
            user.value = userInfo

            return true
        } catch (e: any) {
            error.value = e.response?.data?.message || 'Login failed'
            return false
        } finally {
            loading.value = false
        }
    }

    function logout() {
        token.value = null
        user.value = null
        localStorage.removeItem('token')
    }

    async function fetchUser() {
        if (!token.value) return

        try {
            const userInfo = await authApi.me()
            user.value = userInfo
        } catch {
            // Token is invalid
            logout()
        }
    }

    return {
        token,
        user,
        loading,
        error,
        isAuthenticated,
        login,
        logout,
        fetchUser
    }
})
