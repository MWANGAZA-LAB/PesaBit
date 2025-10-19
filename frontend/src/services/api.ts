import axios, { AxiosInstance, AxiosResponse } from 'axios'
import { User } from '../stores/authStore'

// API Base URL - in production, this would come from environment variables
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000'

// API Response types
export interface RegisterResponse {
  verification_token: string
  expires_at: string
}

export interface VerifyOtpResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  user: User
}

export interface LoginResponse {
  access_token: string
  refresh_token: string
  expires_in: number
  user: User
}

export interface RefreshTokenResponse {
  access_token: string
  expires_in: number
}

export interface WalletBalance {
  bitcoin_balance: number
  mpesa_balance: number
  pending_deposits: number
  pending_withdrawals: number
}

export interface Transaction {
  id: string
  type: 'mpesa_deposit' | 'mpesa_withdrawal' | 'lightning_send' | 'lightning_receive'
  amount: number
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled'
  created_at: string
  completed_at?: string
  description?: string
  external_reference?: string
  fee_amount?: number
}

export interface CreateInvoiceRequest {
  amount: number
  description?: string
  expiry_seconds?: number
}

export interface Invoice {
  invoice_id: string
  bolt11: string
  amount: number
  description?: string
  expires_at: string
  status: 'pending' | 'paid' | 'expired'
}

export interface MpesaDepositRequest {
  amount: number
  phone_number: string
}

export interface MpesaWithdrawalRequest {
  amount: number
  phone_number: string
}

export interface SendPaymentRequest {
  bolt11: string
}

class ApiClient {
  private client: AxiosInstance

  constructor() {
    this.client = axios.create({
      baseURL: API_BASE_URL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    // Request interceptor for auth token
    this.client.interceptors.request.use(
      (config) => {
        // Token will be set via setAuthToken method
        return config
      },
      (error) => {
        return Promise.reject(error)
      }
    )

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response: AxiosResponse) => response.data,
      (error) => {
        // Handle common errors
        if (error.response) {
          const { status, data } = error.response
          
          // Create user-friendly error messages
          switch (status) {
            case 400:
              throw new Error(data?.message || 'Invalid request')
            case 401:
              throw new Error('Session expired. Please log in again')
            case 403:
              throw new Error('Access denied')
            case 404:
              throw new Error('Resource not found')
            case 429:
              throw new Error('Too many requests. Please try again later')
            case 500:
              throw new Error('Server error. Please try again')
            default:
              throw new Error(data?.message || `HTTP ${status} error`)
          }
        } else if (error.request) {
          throw new Error('Network error. Please check your connection')
        } else {
          throw new Error('Request failed')
        }
      }
    )
  }

  // Set authentication token for all requests
  setAuthToken(token: string) {
    this.client.defaults.headers.common['Authorization'] = `Bearer ${token}`
  }

  // Clear authentication token
  clearAuthToken() {
    delete this.client.defaults.headers.common['Authorization']
  }

  // Authentication endpoints
  async register(phoneNumber: string): Promise<RegisterResponse> {
    return this.client.post('/auth/register', { phone_number: phoneNumber })
  }

  async verifyOtp(data: {
    verification_token: string
    otp_code: string
    pin: string
    full_name?: string
    lightning_username: string
  }): Promise<VerifyOtpResponse> {
    return this.client.post('/auth/verify-otp', data)
  }

  async login(phoneNumber: string, pin: string): Promise<LoginResponse> {
    return this.client.post('/auth/login', {
      phone_number: phoneNumber,
      pin,
    })
  }

  async refreshToken(refreshToken: string): Promise<RefreshTokenResponse> {
    return this.client.post('/auth/refresh', {
      refresh_token: refreshToken,
    })
  }

  async logout(): Promise<void> {
    return this.client.post('/auth/logout')
  }

  // User profile endpoints
  async getCurrentUser(): Promise<User> {
    return this.client.get('/users/profile')
  }

  async updateProfile(data: { full_name?: string }): Promise<User> {
    return this.client.patch('/users/profile', data)
  }

  // Wallet endpoints
  async getWalletBalance(): Promise<WalletBalance> {
    return this.client.get('/wallet/balance')
  }

  async getTransactionHistory(limit: number = 50, offset: number = 0): Promise<Transaction[]> {
    return this.client.get('/wallet/transactions', {
      params: { limit, offset },
    })
  }

  // Lightning Network endpoints
  async createInvoice(data: CreateInvoiceRequest): Promise<Invoice> {
    return this.client.post('/lightning/invoices', data)
  }

  async getInvoice(invoiceId: string): Promise<Invoice> {
    return this.client.get(`/lightning/invoices/${invoiceId}`)
  }

  async sendPayment(data: SendPaymentRequest): Promise<Transaction> {
    return this.client.post('/lightning/payments', data)
  }

  // M-Pesa endpoints
  async initiateDeposit(data: MpesaDepositRequest): Promise<Transaction> {
    return this.client.post('/mpesa/deposit', data)
  }

  async initiateWithdrawal(data: MpesaWithdrawalRequest): Promise<Transaction> {
    return this.client.post('/mpesa/withdrawal', data)
  }

  // Get transaction by ID
  async getTransaction(transactionId: string): Promise<Transaction> {
    return this.client.get(`/transactions/${transactionId}`)
  }
}

// Export singleton instance
export const api = new ApiClient()