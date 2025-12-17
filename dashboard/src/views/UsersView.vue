<script setup lang="ts">
import { ref, onMounted } from 'vue'
import MainLayout from '@/layouts/MainLayout.vue'
import Modal from '@/components/Modal.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import { usersApi, type User, type CreateUserRequest, type UpdateUserRequest, type ChangePasswordRequest } from '@/services/users'

const users = ref<User[]>([])
const loading = ref(false)
const saving = ref(false)
const error = ref('')

// Modals
const showAddModal = ref(false)
const showEditModal = ref(false)
const showPasswordModal = ref(false)
const showDeleteConfirm = ref(false)

// Currently editing
const editingUser = ref<User | null>(null)
const deletingUser = ref<User | null>(null)

// Forms
const addForm = ref<CreateUserRequest>({
  username: '',
  password: ''
})

const editForm = ref<UpdateUserRequest>({
  username: ''
})

const passwordForm = ref<ChangePasswordRequest>({
  current_password: '',
  new_password: ''
})

const confirmPassword = ref('')

// Load users
const fetchUsers = async () => {
  loading.value = true
  error.value = ''
  try {
    users.value = await usersApi.getUsers()
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to load users'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchUsers()
})

// Add user
const openAddModal = () => {
  addForm.value = { username: '', password: '' }
  confirmPassword.value = ''
  error.value = ''
  showAddModal.value = true
}

const saveNewUser = async () => {
  if (addForm.value.password !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  if (addForm.value.password.length < 6) {
    error.value = 'Password must be at least 6 characters'
    return
  }
  
  saving.value = true
  error.value = ''
  try {
    await usersApi.createUser(addForm.value)
    showAddModal.value = false
    await fetchUsers()
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to create user'
  } finally {
    saving.value = false
  }
}

// Edit user
const openEditModal = (user: User) => {
  editingUser.value = user
  editForm.value = { username: user.username }
  error.value = ''
  showEditModal.value = true
}

const saveEditUser = async () => {
  if (!editingUser.value) return
  
  saving.value = true
  error.value = ''
  try {
    await usersApi.updateUser(editingUser.value.id, editForm.value)
    showEditModal.value = false
    editingUser.value = null
    await fetchUsers()
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to update user'
  } finally {
    saving.value = false
  }
}

// Change password
const openPasswordModal = (user: User) => {
  editingUser.value = user
  passwordForm.value = { current_password: '', new_password: '' }
  confirmPassword.value = ''
  error.value = ''
  showPasswordModal.value = true
}

const savePassword = async () => {
  if (!editingUser.value) return
  
  if (passwordForm.value.new_password !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }
  if (passwordForm.value.new_password.length < 6) {
    error.value = 'Password must be at least 6 characters'
    return
  }
  
  saving.value = true
  error.value = ''
  try {
    await usersApi.changePassword(editingUser.value.id, passwordForm.value)
    showPasswordModal.value = false
    editingUser.value = null
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to change password'
  } finally {
    saving.value = false
  }
}

// Delete user
const confirmDelete = (user: User) => {
  deletingUser.value = user
  showDeleteConfirm.value = true
}

const executeDelete = async () => {
  if (!deletingUser.value) return
  
  try {
    await usersApi.deleteUser(deletingUser.value.id)
    showDeleteConfirm.value = false
    deletingUser.value = null
    await fetchUsers()
  } catch (e: any) {
    error.value = e.response?.data?.message || 'Failed to delete user'
    showDeleteConfirm.value = false
  }
}

// Format date
const formatDate = (timestamp: number) => {
  return new Date(timestamp * 1000).toLocaleString()
}
</script>

<template>
  <MainLayout>
    <template #title>User Management</template>
    
    <div class="space-y-6">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <p class="text-slate-400">Manage system users. All users have full access.</p>
        </div>
        <button @click="openAddModal" class="btn-primary flex items-center gap-2">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Add User
        </button>
      </div>

      <!-- Error message -->
      <div v-if="error && !showAddModal && !showEditModal && !showPasswordModal" class="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
        <p class="text-red-400 text-sm">{{ error }}</p>
      </div>

      <!-- Loading state -->
      <div v-if="loading" class="glass-card p-8 text-center">
        <svg class="animate-spin h-8 w-8 mx-auto text-cyan-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <p class="mt-2 text-slate-400">Loading users...</p>
      </div>

      <!-- Users Table -->
      <div v-else class="glass-card overflow-hidden">
        <table class="w-full">
          <thead class="bg-slate-800/50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase">Username</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase">Type</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase">Created</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase">Updated</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-slate-400 uppercase">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-700/50">
            <tr v-for="user in users" :key="user.id" class="hover:bg-slate-800/30">
              <td class="px-6 py-4">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 bg-gradient-to-br from-cyan-500 to-blue-600 rounded-full flex items-center justify-center">
                    <span class="text-white font-semibold text-sm">{{ user.username.charAt(0).toUpperCase() }}</span>
                  </div>
                  <span class="font-medium text-white">{{ user.username }}</span>
                </div>
              </td>
              <td class="px-6 py-4">
                <span v-if="user.is_default" class="px-2 py-1 rounded text-xs font-medium text-amber-400 bg-amber-500/20">
                  Default
                </span>
                <span v-else class="px-2 py-1 rounded text-xs font-medium text-slate-400 bg-slate-500/20">
                  Regular
                </span>
              </td>
              <td class="px-6 py-4 text-sm text-slate-400">
                {{ formatDate(user.created_at) }}
              </td>
              <td class="px-6 py-4 text-sm text-slate-400">
                {{ formatDate(user.updated_at) }}
              </td>
              <td class="px-6 py-4">
                <div class="flex items-center gap-2">
                  <button @click="openEditModal(user)" class="text-cyan-400 hover:text-cyan-300 p-1" title="Edit username">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button @click="openPasswordModal(user)" class="text-purple-400 hover:text-purple-300 p-1" title="Change password">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                    </svg>
                  </button>
                  <button 
                    v-if="!user.is_default"
                    @click="confirmDelete(user)" 
                    class="text-red-400 hover:text-red-300 p-1" 
                    title="Delete user"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  <span v-else class="text-slate-600 p-1" title="Default user cannot be deleted">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                    </svg>
                  </span>
                </div>
              </td>
            </tr>
            <tr v-if="!users.length">
              <td colspan="5" class="px-6 py-8 text-center text-slate-400">
                No users found. Click "Add User" to create one.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Add User Modal -->
    <Modal :show="showAddModal" title="Add User" size="md" @close="showAddModal = false">
      <form @submit.prevent="saveNewUser" class="space-y-4">
        <div v-if="error" class="p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p class="text-red-400 text-sm">{{ error }}</p>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Username</label>
          <input v-model="addForm.username" type="text" class="input-dark w-full" placeholder="Enter username" required />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Password</label>
          <input v-model="addForm.password" type="password" class="input-dark w-full" placeholder="Enter password (min 6 chars)" required />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Confirm Password</label>
          <input v-model="confirmPassword" type="password" class="input-dark w-full" placeholder="Confirm password" required />
        </div>
      </form>
      <template #footer>
        <button @click="showAddModal = false" class="btn-secondary">Cancel</button>
        <button @click="saveNewUser" :disabled="saving" class="btn-primary">
          {{ saving ? 'Creating...' : 'Create User' }}
        </button>
      </template>
    </Modal>

    <!-- Edit User Modal -->
    <Modal :show="showEditModal" title="Edit User" size="md" @close="showEditModal = false">
      <form @submit.prevent="saveEditUser" class="space-y-4">
        <div v-if="error" class="p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p class="text-red-400 text-sm">{{ error }}</p>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Username</label>
          <input v-model="editForm.username" type="text" class="input-dark w-full" placeholder="Enter username" required />
        </div>
      </form>
      <template #footer>
        <button @click="showEditModal = false" class="btn-secondary">Cancel</button>
        <button @click="saveEditUser" :disabled="saving" class="btn-primary">
          {{ saving ? 'Saving...' : 'Save Changes' }}
        </button>
      </template>
    </Modal>

    <!-- Change Password Modal -->
    <Modal :show="showPasswordModal" title="Change Password" size="md" @close="showPasswordModal = false">
      <form @submit.prevent="savePassword" class="space-y-4">
        <div v-if="error" class="p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <p class="text-red-400 text-sm">{{ error }}</p>
        </div>
        <p class="text-sm text-slate-400">
          Changing password for: <span class="text-white font-medium">{{ editingUser?.username }}</span>
        </p>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Current Password</label>
          <input v-model="passwordForm.current_password" type="password" class="input-dark w-full" placeholder="Enter current password" />
          <p class="text-xs text-slate-500 mt-1">Leave empty if changing as admin</p>
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">New Password</label>
          <input v-model="passwordForm.new_password" type="password" class="input-dark w-full" placeholder="Enter new password (min 6 chars)" required />
        </div>
        <div>
          <label class="block text-sm font-medium text-slate-300 mb-2">Confirm New Password</label>
          <input v-model="confirmPassword" type="password" class="input-dark w-full" placeholder="Confirm new password" required />
        </div>
      </form>
      <template #footer>
        <button @click="showPasswordModal = false" class="btn-secondary">Cancel</button>
        <button @click="savePassword" :disabled="saving" class="btn-primary">
          {{ saving ? 'Changing...' : 'Change Password' }}
        </button>
      </template>
    </Modal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      title="Delete User"
      :message="`Are you sure you want to delete user '${deletingUser?.username}'? This action cannot be undone.`"
      type="danger"
      confirm-text="Delete"
      @confirm="executeDelete"
      @cancel="showDeleteConfirm = false"
    />
  </MainLayout>
</template>

<style scoped></style>
