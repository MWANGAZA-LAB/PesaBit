import React, { useState, useEffect } from 'react'
import { useAuthStore } from '../stores/authStore'
import { api, WalletBalance, Transaction } from '../services/api'
import { toast } from 'react-hot-toast'

export const DashboardPage: React.FC = () => {
  const { user, logout } = useAuthStore()
  const [balance, setBalance] = useState<WalletBalance | null>(null)
  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [loading, setLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<'overview' | 'send' | 'receive' | 'history'>('overview')

  // Load dashboard data
  useEffect(() => {
    loadDashboardData()
  }, [])

  const loadDashboardData = async () => {
    try {
      setLoading(true)
      
      const [balanceData, transactionData] = await Promise.all([
        api.getWalletBalance(),
        api.getTransactionHistory(10, 0)
      ])
      
      setBalance(balanceData)
      setTransactions(transactionData)
    } catch (error: any) {
      toast.error(error.message || 'Failed to load dashboard data')
    } finally {
      setLoading(false)
    }
  }

  const formatAmount = (amount: number) => {
    return new Intl.NumberFormat('en-KE', {
      style: 'currency',
      currency: 'KES',
      minimumFractionDigits: 2,
    }).format(amount)
  }

  const formatBitcoin = (sats: number) => {
    return `${(sats / 100000000).toFixed(8)} BTC`
  }

  const getStatusColor = (status: Transaction['status']) => {
    switch (status) {
      case 'completed': return 'text-green-600 bg-green-100'
      case 'pending': return 'text-yellow-600 bg-yellow-100'
      case 'processing': return 'text-blue-600 bg-blue-100'
      case 'failed': return 'text-red-600 bg-red-100'
      case 'cancelled': return 'text-gray-600 bg-gray-100'
      default: return 'text-gray-600 bg-gray-100'
    }
  }

  const getTransactionIcon = (type: Transaction['type']) => {
    switch (type) {
      case 'mpesa_deposit':
        return (
          <div className="w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
            <svg className="w-4 h-4 text-green-600" fill="currentColor" viewBox="0 0 24 24">
              <path d="M7 14l5-5 5 5z"/>
            </svg>
          </div>
        )
      case 'mpesa_withdrawal':
        return (
          <div className="w-8 h-8 bg-red-100 rounded-full flex items-center justify-center">
            <svg className="w-4 h-4 text-red-600" fill="currentColor" viewBox="0 0 24 24">
              <path d="M7 10l5 5 5-5z"/>
            </svg>
          </div>
        )
      case 'lightning_send':
        return (
          <div className="w-8 h-8 bg-orange-100 rounded-full flex items-center justify-center">
            <svg className="w-4 h-4 text-orange-600" fill="currentColor" viewBox="0 0 24 24">
              <path d="M13 2.05v3.03c3.39.49 6 3.39 6 6.92 0 .9-.18 1.75-.48 2.54l2.6 1.53c.56-1.24.88-2.62.88-4.07 0-5.18-3.95-9.45-9-9.95z"/>
            </svg>
          </div>
        )
      case 'lightning_receive':
        return (
          <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
            <svg className="w-4 h-4 text-blue-600" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 2.05c5.05.5 9 4.76 9 9.95 0 1.45-.32 2.83-.88 4.07l-2.6-1.53c.3-.79.48-1.64.48-2.54 0-3.53-2.61-6.43-6-6.92V2.05z"/>
            </svg>
          </div>
        )
      default:
        return (
          <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
            <svg className="w-4 h-4 text-gray-600" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"/>
            </svg>
          </div>
        )
    }
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-4 border-green-500 border-t-transparent mx-auto mb-4"></div>
          <p className="text-gray-600">Loading dashboard...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm">
        <div className="max-w-6xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
              <p className="text-gray-600">
                Welcome back, {user?.full_name || user?.lightning_username}
              </p>
            </div>
            
            <div className="flex items-center space-x-4">
              <div className="text-right">
                <p className="text-sm text-gray-600">Lightning Address</p>
                <p className="font-mono text-sm text-green-600">
                  {user?.lightning_address}
                </p>
              </div>
              
              <button
                onClick={logout}
                className="text-gray-400 hover:text-gray-600 p-2 rounded-lg hover:bg-gray-100 transition-colors"
                title="Logout"
              >
                <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M17 7l-1.41 1.41L18.17 11H8v2h10.17l-2.58 2.59L17 17l5-5zM4 5h8V3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h8v-2H4V5z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Balance Cards */}
      <div className="max-w-6xl mx-auto px-4 py-6">
        <div className="grid md:grid-cols-2 gap-6 mb-8">
          {/* Bitcoin Balance */}
          <div className="bg-gradient-to-r from-orange-400 to-orange-600 rounded-xl p-6 text-white">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">Bitcoin Balance</h3>
              <svg className="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M17.8 19.2L16.4 20.6C15.3 19.5 14.2 18.7 12.9 18.1C11.2 17.3 9.3 16.9 7.5 16.9C5.8 16.9 4.2 17.2 2.8 17.7L2 16.1C3.7 15.5 5.6 15.2 7.5 15.2C9.7 15.2 11.8 15.7 13.6 16.6C14.6 17.1 15.5 17.7 16.3 18.4L17.8 19.2M12 2C6.5 2 2 6.5 2 12S6.5 22 12 22 22 17.5 22 12 17.5 2 12 2M12 4C16.4 4 20 7.6 20 12S16.4 20 12 20 4 16.4 4 12 7.6 4 12 4Z"/>
              </svg>
            </div>
            <p className="text-3xl font-bold mb-2">
              {balance ? formatBitcoin(balance.bitcoin_balance) : '---'}
            </p>
            <p className="text-orange-200">
              ≈ {balance ? formatAmount(balance.bitcoin_balance * 0.01) : '---'}
            </p>
          </div>

          {/* M-Pesa Balance */}
          <div className="bg-gradient-to-r from-green-400 to-green-600 rounded-xl p-6 text-white">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">M-Pesa Balance</h3>
              <svg className="w-8 h-8" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z"/>
              </svg>
            </div>
            <p className="text-3xl font-bold mb-2">
              {balance ? formatAmount(balance.mpesa_balance) : '---'}
            </p>
            <p className="text-green-200">Available to withdraw</p>
          </div>
        </div>

        {/* Navigation Tabs */}
        <div className="bg-white rounded-lg shadow-sm mb-6">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8 px-6">
              {[
                { id: 'overview', name: 'Overview', icon: '📊' },
                { id: 'send', name: 'Send', icon: '⚡' },
                { id: 'receive', name: 'Receive', icon: '📥' },
                { id: 'history', name: 'History', icon: '📜' },
              ].map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as any)}
                  className={`py-4 px-2 border-b-2 font-medium text-sm transition-colors ${
                    activeTab === tab.id
                      ? 'border-green-500 text-green-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  <span className="mr-2">{tab.icon}</span>
                  {tab.name}
                </button>
              ))}
            </nav>
          </div>

          {/* Tab Content */}
          <div className="p-6">
            {activeTab === 'overview' && (
              <div>
                <h3 className="text-lg font-semibold mb-4">Recent Transactions</h3>
                
                {transactions.length === 0 ? (
                  <div className="text-center py-8 text-gray-500">
                    <svg className="w-16 h-16 mx-auto mb-4 text-gray-300" fill="currentColor" viewBox="0 0 24 24">
                      <path d="M9,11H7v2h2V11z M13,11h-2v2h2V11z M17,11h-2v2h2V11z M19,4H18V2h-2v2H8V2H6v2H5C3.89,4 3.01,4.9 3.01,6L3,20 c0,1.1 0.89,2 2,2H19c1.1,0 2-0.9 2-2V6C21,4.9 20.1,4 19,4z"/>
                    </svg>
                    <p>No transactions yet</p>
                    <p className="text-sm">Start by making a deposit or sending a payment</p>
                  </div>
                ) : (
                  <div className="space-y-3">
                    {transactions.slice(0, 5).map((tx) => (
                      <div key={tx.id} className="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                        <div className="flex items-center space-x-3">
                          {getTransactionIcon(tx.type)}
                          <div>
                            <p className="font-medium text-gray-900 capitalize">
                              {tx.type.replace('_', ' ')}
                            </p>
                            <p className="text-sm text-gray-600">
                              {new Date(tx.created_at).toLocaleDateString()}
                            </p>
                          </div>
                        </div>
                        <div className="text-right">
                          <p className="font-semibold">
                            {formatAmount(tx.amount)}
                          </p>
                          <span className={`text-xs px-2 py-1 rounded-full ${getStatusColor(tx.status)}`}>
                            {tx.status}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {activeTab === 'send' && (
              <div className="text-center py-8 text-gray-500">
                <p>Send payment functionality coming soon!</p>
              </div>
            )}

            {activeTab === 'receive' && (
              <div className="text-center py-8 text-gray-500">
                <p>Receive payment functionality coming soon!</p>
              </div>
            )}

            {activeTab === 'history' && (
              <div className="text-center py-8 text-gray-500">
                <p>Transaction history view coming soon!</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}