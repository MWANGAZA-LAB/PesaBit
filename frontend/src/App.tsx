import React, { useEffect } from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './stores/authStore'

// Pages
import { LandingPage } from './pages/LandingPage'
import { RegisterPage } from './pages/RegisterPage' 
import { LoginPage } from './pages/LoginPage'
import { DashboardPage } from './pages/DashboardPage'

// Components
import { ProtectedRoute } from './components/ProtectedRoute'
import { LoadingScreen } from './components/LoadingScreen'

function App() {
  const { isAuthenticated, isLoading, checkAuthStatus } = useAuthStore()

  // Check authentication status on app start
  useEffect(() => {
    checkAuthStatus()
  }, [checkAuthStatus])

  // Show loading screen while checking auth
  if (isLoading) {
    return <LoadingScreen />
  }

  return (
    <div className="App min-h-screen">
      <Routes>
        {/* Public routes */}
        <Route 
          path="/" 
          element={isAuthenticated ? <Navigate to="/dashboard" replace /> : <LandingPage />} 
        />
        <Route 
          path="/register" 
          element={isAuthenticated ? <Navigate to="/dashboard" replace /> : <RegisterPage />} 
        />
        <Route 
          path="/login" 
          element={isAuthenticated ? <Navigate to="/dashboard" replace /> : <LoginPage />} 
        />

        {/* Protected routes (require authentication) */}
        <Route 
          path="/dashboard" 
          element={
            <ProtectedRoute>
              <DashboardPage />
            </ProtectedRoute>
          } 
        />

        {/* Catch-all route */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </div>
  )
}

export default App