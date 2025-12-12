import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/login',
            name: 'login',
            component: () => import('@/views/LoginView.vue'),
            meta: { requiresAuth: false }
        },
        {
            path: '/',
            name: 'dashboard',
            component: () => import('@/views/DashboardView.vue'),
            meta: { requiresAuth: true }
        },
        {
            path: '/config',
            name: 'config',
            component: () => import('@/views/ConfigView.vue'),
            meta: { requiresAuth: true }
        }
    ]
})

// Navigation guard
router.beforeEach((to, from, next) => {
    const authStore = useAuthStore()

    if (to.meta.requiresAuth && !authStore.isAuthenticated) {
        next({ name: 'login' })
    } else if (to.name === 'login' && authStore.isAuthenticated) {
        next({ name: 'dashboard' })
    } else {
        next()
    }
})

export default router
