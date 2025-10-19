import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { api } from '../services/api'

export interface User {
  id: string
  phone_number: string
  lightning_username: string
  lightning_address: string
  full_name?: string
  kyc_status: 'none' | 'pending' | 'verified' | 'rejected'
  kyc_tier: 'tier0' | 'tier1' | 'tier2'
  created_at: string
}

interface AuthTokens {
  access_token: string
  refresh_token: string
}

interface VerifyOtpData {
  verification_token: string
  otp_code: string
  pin: string
  full_name?: string
  lightning_username: string
}

interface UpdateProfileData {
  full_name?: string
}

interface AuthState {
  // State
  isAuthenticated: boolean
  isLoading: boolean
  user: User | null
  accessToken: string | null
  refreshToken: string | null

  // Actions
  setAuth: (tokens: AuthTokens, user: User) => void
  logout: () => void
  checkAuthStatus: () => Promise<void>
  refreshAuth: () => Promise<boolean>
  register: (phoneNumber: string) => Promise<{ verification_token: string }>
  verifyOtp: (data: VerifyOtpData) => Promise<void>
  login: (phoneNumber: string, pin: string) => Promise<void>
  updateProfile: (data: UpdateProfileData) => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      // Initial state
      isAuthenticated: false,
      isLoading: true,
      user: null,
      accessToken: null,
      refreshToken: null,

      // Set authentication tokens and user data
      setAuth: (tokens: AuthTokens, user: User) => {
        // Update API client with new token
        api.setAuthToken(tokens.access_token)
        
        set({
          isAuthenticated: true,
          isLoading: false,
          user,
          accessToken: tokens.access_token,
          refreshToken: tokens.refresh_token,
        })
      },

      // Clear all authentication data
      logout: () => {
        api.clearAuthToken()
        
        set({
          isAuthenticated: false,
          isLoading: false,
          user: null,
          accessToken: null,
          refreshToken: null,
        })
      },

      // Check if current tokens are valid
      checkAuthStatus: async () => {
        const { accessToken, refreshToken } = get()
        
        if (!accessToken || !refreshToken) {
          set({ isAuthenticated: false, isLoading: false })
          return
        }

        try {
          // Set token for API calls
          api.setAuthToken(accessToken)
          
          // Try to get current user profile (validates token)
          const user = await api.getCurrentUser()
          
          set({
            isAuthenticated: true,
            isLoading: false,
            user,
          })
        } catch (error: any) {
          // Token might be expired, try to refresh
          if (error.response?.status === 401) {
            const refreshed = await get().refreshAuth()
            if (!refreshed) {
              get().logout()
            }
          } else {
            get().logout()
          }
        }
      },

      // Refresh authentication tokens
      refreshAuth: async () => {
        const { refreshToken } = get()
        
        if (!refreshToken) {
          return false
        }

        try {
          const response = await api.refreshToken(refreshToken)
          
          // Update access token
          api.setAuthToken(response.access_token)
          
          set({
            accessToken: response.access_token,
            isAuthenticated: true,
            isLoading: false,
          })
          
          return true
        } catch (error) {
          console.error('Token refresh failed:', error)
          return false
        }
      },

      // Register new user with phone number
      register: async (phoneNumber: string) => {
        set({ isLoading: true })
        
        try {
          const response = await api.register(phoneNumber)
          
          set({ isLoading: false })
          return response
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      // Verify OTP and complete registration
      verifyOtp: async (data: VerifyOtpData) => {
        set({ isLoading: true })
        
        try {
          const response = await api.verifyOtp(data)
          
          // Authentication successful
          get().setAuth(
            {
              access_token: response.access_token,
              refresh_token: response.refresh_token,
            },
            response.user
          )
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      // Login with phone and PIN
      login: async (phoneNumber: string, pin: string) => {
        set({ isLoading: true })
        
        try {
          const response = await api.login(phoneNumber, pin)
          
          // Authentication successful
          get().setAuth(
            {
              access_token: response.access_token,
              refresh_token: response.refresh_token,
            },
            response.user
          )
        } catch (error) {
          set({ isLoading: false })
          throw error
        }
      },

      // Update user profile
      updateProfile: async (data: UpdateProfileData) => {
        try {
          const updatedUser = await api.updateProfile(data)
          
          set((state: AuthState) => ({
            ...state,
            user: updatedUser
          }))
        } catch (error) {
          throw error
        }
      },
    }),
    {
      name: 'pesa-auth',
      partialize: (state: AuthState) => ({
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
)