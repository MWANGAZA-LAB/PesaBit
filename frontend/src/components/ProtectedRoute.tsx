import React from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '../stores/authStore'
import { LoadingScreen } from './LoadingScreen'

interface ProtectedRouteProps {
  children: React.ReactNode
  requireAuth?: boolean
}

export const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ 
  children, 
  requireAuth = true 
}) => {
  const location = useLocation()
  const { isAuthenticated, isLoading } = useAuthStore()

  // Show loading screen while checking auth status
  if (isLoading) {
    return <LoadingScreen message="Checking authentication..." />
  }

  // If route requires authentication but user is not authenticated
  if (requireAuth && !isAuthenticated) {
    // Save attempted location for redirect after login
    return <Navigate to="/login" state={{ from: location }} replace />
  }

  // If route is for unauthenticated users but user is authenticated
  if (!requireAuth && isAuthenticated) {
    // Redirect to dashboard if user is already logged in
    return <Navigate to="/dashboard" replace />
  }

  return <>{children}</>
}