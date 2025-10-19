'use client';

import { useState } from 'react';
import { Zap, ArrowLeft, Phone, Lock, Mail } from 'lucide-react';

export default function RegisterPage() {
  const [phoneNumber, setPhoneNumber] = useState('');
  const [email, setEmail] = useState('');
  const [pin, setPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleRegister = async (e) => {
    e.preventDefault();
    setError('');

    if (!phoneNumber || !email || !pin || !confirmPin) {
      setError('Please fill in all fields');
      return;
    }

    if (pin.length < 4) {
      setError('PIN must be at least 4 digits');
      return;
    }

    if (pin !== confirmPin) {
      setError('PINs do not match');
      return;
    }

    setLoading(true);
    try {
      const response = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          phoneNumber: `+254${phoneNumber}`,
          email,
          pin,
        }),
      });

      if (!response.ok) {
        const data = await response.json();
        throw new Error(data.message || 'Registration failed');
      }

      window.location.href = '/auth/verify-otp';
    } catch (err) {
      console.error(err);
      setError(err.message || 'Registration failed. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-orange-50 to-green-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b border-gray-100">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-gradient-to-r from-orange-500 to-green-500 rounded-lg flex items-center justify-center">
                <Zap className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold text-gray-900">PesaBit</span>
            </div>
            <a
              href="/"
              className="text-gray-600 hover:text-gray-900 font-medium flex items-center space-x-1"
            >
              <ArrowLeft className="w-4 h-4" />
              <span>Back</span>
            </a>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="max-w-md mx-auto px-4 sm:px-6 lg:px-8 py-16">
        <div className="bg-white rounded-xl shadow-lg border border-gray-100 p-8">
          {/* Title */}
          <div className="text-center mb-8">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">Create Account</h1>
            <p className="text-gray-600">Join PesaBit and start sending money instantly</p>
          </div>

          {/* Error Message */}
          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
              <p className="text-red-800 text-sm">{error}</p>
            </div>
          )}

          {/* Registration Form */}
          <form onSubmit={handleRegister} className="space-y-4">
            {/* Phone Number */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Phone Number
              </label>
              <div className="flex rounded-lg border border-gray-300 overflow-hidden focus-within:border-orange-500 focus-within:ring-2 focus-within:ring-orange-200 transition">
                <div className="bg-gray-50 px-3 py-3 flex items-center border-r border-gray-300">
                  <Phone className="w-5 h-5 text-gray-400 mr-2" />
                  <span className="text-gray-600 text-sm">+254</span>
                </div>
                <input
                  type="tel"
                  placeholder="712 345 678"
                  value={phoneNumber}
                  onChange={(e) => setPhoneNumber(e.target.value.replace(/\D/g, ''))}
                  maxLength="9"
                  className="flex-1 px-4 py-3 focus:outline-none"
                />
              </div>
              <p className="text-xs text-gray-500 mt-1">9 digits only</p>
            </div>

            {/* Email */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Email Address
              </label>
              <div className="flex rounded-lg border border-gray-300 overflow-hidden focus-within:border-orange-500 focus-within:ring-2 focus-within:ring-orange-200 transition">
                <div className="bg-gray-50 px-3 py-3 flex items-center border-r border-gray-300">
                  <Mail className="w-5 h-5 text-gray-400" />
                </div>
                <input
                  type="email"
                  placeholder="you@example.com"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  className="flex-1 px-4 py-3 focus:outline-none"
                />
              </div>
            </div>

            {/* PIN */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Create PIN (like M-Pesa)
              </label>
              <div className="flex rounded-lg border border-gray-300 overflow-hidden focus-within:border-orange-500 focus-within:ring-2 focus-within:ring-orange-200 transition">
                <div className="bg-gray-50 px-3 py-3 flex items-center border-r border-gray-300">
                  <Lock className="w-5 h-5 text-gray-400" />
                </div>
                <input
                  type="password"
                  placeholder="••••"
                  value={pin}
                  onChange={(e) => setPin(e.target.value.replace(/\D/g, ''))}
                  maxLength="6"
                  className="flex-1 px-4 py-3 focus:outline-none"
                />
              </div>
              <p className="text-xs text-gray-500 mt-1">4-6 digits</p>
            </div>

            {/* Confirm PIN */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Confirm PIN
              </label>
              <div className="flex rounded-lg border border-gray-300 overflow-hidden focus-within:border-orange-500 focus-within:ring-2 focus-within:ring-orange-200 transition">
                <div className="bg-gray-50 px-3 py-3 flex items-center border-r border-gray-300">
                  <Lock className="w-5 h-5 text-gray-400" />
                </div>
                <input
                  type="password"
                  placeholder="••••"
                  value={confirmPin}
                  onChange={(e) => setConfirmPin(e.target.value.replace(/\D/g, ''))}
                  maxLength="6"
                  className="flex-1 px-4 py-3 focus:outline-none"
                />
              </div>
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              disabled={loading}
              className="w-full bg-gradient-to-r from-orange-500 to-green-500 text-white py-3 rounded-lg font-semibold hover:from-orange-600 hover:to-green-600 transition-all disabled:opacity-50 disabled:cursor-not-allowed mt-6"
            >
              {loading ? 'Creating Account...' : 'Create Account'}
            </button>
          </form>

          {/* Login Link */}
          <div className="text-center mt-6 pt-6 border-t border-gray-200">
            <p className="text-gray-600">
              Already have an account?{' '}
              <a href="/auth/login" className="text-orange-500 font-semibold hover:text-orange-600">
                Sign In
              </a>
            </p>
          </div>

          {/* Benefits */}
          <div className="mt-8 pt-8 border-t border-gray-200 space-y-3">
            <p className="text-xs font-semibold text-gray-700 uppercase tracking-wide">Benefits</p>
            <div className="space-y-2 text-sm text-gray-600">
              <p>✓ Get your <span className="font-medium">name@pesa.co.ke</span> Lightning address</p>
              <p>✓ Instant global payments in under 2 minutes</p>
              <p>✓ Bank-level security with encrypted PIN</p>
            </div>
          </div>
        </div>

        {/* Security Info */}
        <div className="mt-6 text-center text-xs text-gray-500">
          <p>We keep your data secure and encrypted. No fees to sign up.</p>
        </div>
      </div>
    </div>
  );
}