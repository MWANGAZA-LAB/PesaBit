'use client';

import { useState } from 'react';
import { Zap, Smartphone, Shield, Globe, ArrowRight, Bitcoin, Phone } from 'lucide-react';

export default function HomePage() {
  const [phoneNumber, setPhoneNumber] = useState('');

  const handleGetStarted = () => {
    if (phoneNumber) {
      window.location.href = '/auth/register';
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
            <div className="flex items-center space-x-4">
              <button className="text-gray-600 hover:text-gray-900 font-medium">
                Login
              </button>
              <button className="bg-orange-500 text-white px-4 py-2 rounded-lg font-medium hover:bg-orange-600 transition-colors">
                Sign Up
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-16 pb-20">
        <div className="text-center">
          <h1 className="text-4xl md:text-6xl font-bold text-gray-900 mb-6">
            Lightning-Fast Money
            <span className="block text-orange-500">Transfers 🚀</span>
          </h1>
          <p className="text-xl text-gray-600 mb-8 max-w-3xl mx-auto">
            Send money globally in 2 minutes via Bitcoin Lightning Network and M-Pesa. 
            No crypto knowledge needed - just your phone number.
          </p>
          
          {/* Phone Input */}
          <div className="max-w-md mx-auto mb-8">
            <div className="flex rounded-lg shadow-lg overflow-hidden">
              <div className="bg-white px-4 py-3 flex items-center border-r">
                <Phone className="w-5 h-5 text-gray-400 mr-2" />
                <span className="text-gray-600">+254</span>
              </div>
              <input
                type="tel"
                placeholder="712 345 678"
                value={phoneNumber}
                onChange={(e) => setPhoneNumber(e.target.value)}
                className="flex-1 px-4 py-3 text-lg focus:outline-none"
              />
              <button
                onClick={handleGetStarted}
                className="bg-orange-500 text-white px-6 py-3 font-medium hover:bg-orange-600 transition-colors flex items-center"
              >
                Get Started
                <ArrowRight className="w-5 h-5 ml-2" />
              </button>
            </div>
          </div>

          <p className="text-sm text-gray-500">
            Get your own <span className="font-medium">yourname@pesa.co.ke</span> Lightning address
          </p>
        </div>
      </section>

      {/* Features Grid */}
      <section className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pb-20">
        <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
          <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <div className="w-12 h-12 bg-orange-100 rounded-lg flex items-center justify-center mb-4">
              <Zap className="w-6 h-6 text-orange-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Lightning Fast</h3>
            <p className="text-gray-600">Instant Bitcoin payments via Lightning Network in under 2 minutes</p>
          </div>

          <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4">
              <Smartphone className="w-6 h-6 text-green-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Mobile First</h3>
            <p className="text-gray-600">Progressive Web App optimized for smartphones and M-Pesa users</p>
          </div>

          <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4">
              <Shield className="w-6 h-6 text-blue-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Bank-Level Security</h3>
            <p className="text-gray-600">Phone number + PIN authentication with encrypted JWT tokens</p>
          </div>

          <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-100">
            <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4">
              <Globe className="w-6 h-6 text-purple-500" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">Global Reach</h3>
            <p className="text-gray-600">Send and receive money anywhere in the world instantly</p>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="bg-white py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-3xl font-bold text-gray-900 mb-4">How It Works</h2>
            <p className="text-xl text-gray-600">Simple as M-Pesa, powered by Bitcoin Lightning</p>
          </div>

          <div className="grid md:grid-cols-5 gap-8 items-center">
            <div className="text-center">
              <div className="w-16 h-16 bg-orange-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span className="text-2xl font-bold text-orange-500">1</span>
              </div>
              <h3 className="font-semibold text-gray-900 mb-2">Sign Up</h3>
              <p className="text-sm text-gray-600">Register with your phone number like M-Pesa</p>
            </div>

            <ArrowRight className="w-6 h-6 text-gray-300 mx-auto hidden md:block" />

            <div className="text-center">
              <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <span className="text-2xl font-bold text-green-500">2</span>
              </div>
              <h3 className="font-semibold text-gray-900 mb-2">Deposit</h3>
              <p className="text-sm text-gray-600">Add money via M-Pesa STK Push</p>
            </div>

            <ArrowRight className="w-6 h-6 text-gray-300 mx-auto hidden md:block" />

            <div className="text-center">
              <div className="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Bitcoin className="w-8 h-8 text-blue-500" />
              </div>
              <h3 className="font-semibold text-gray-900 mb-2">Send Globally</h3>
              <p className="text-sm text-gray-600">Instant Lightning payments worldwide</p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="bg-gradient-to-r from-orange-500 to-green-500 py-16">
        <div className="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl font-bold text-white mb-4">
            Ready to Send Money at Lightning Speed?
          </h2>
          <p className="text-xl text-orange-100 mb-8">
            Join thousands of users already using PesaBit for instant global payments
          </p>
          <button className="bg-white text-orange-500 px-8 py-4 rounded-lg font-semibold text-lg hover:bg-gray-50 transition-colors">
            Get Started Now
          </button>
        </div>
      </section>

      {/* Footer */}
      <footer className="bg-gray-900 text-white py-12">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="grid md:grid-cols-4 gap-8">
            <div>
              <div className="flex items-center space-x-2 mb-4">
                <div className="w-8 h-8 bg-gradient-to-r from-orange-500 to-green-500 rounded-lg flex items-center justify-center">
                  <Zap className="w-5 h-5 text-white" />
                </div>
                <span className="text-xl font-bold">PesaBit</span>
              </div>
              <p className="text-gray-400">
                Lightning-fast money transfers via Bitcoin and M-Pesa
              </p>
            </div>
            
            <div>
              <h3 className="font-semibold mb-4">Product</h3>
              <ul className="space-y-2 text-gray-400">
                <li><a href="#" className="hover:text-white">Features</a></li>
                <li><a href="#" className="hover:text-white">Pricing</a></li>
                <li><a href="#" className="hover:text-white">API Docs</a></li>
              </ul>
            </div>
            
            <div>
              <h3 className="font-semibold mb-4">Company</h3>
              <ul className="space-y-2 text-gray-400">
                <li><a href="#" className="hover:text-white">About</a></li>
                <li><a href="#" className="hover:text-white">Security</a></li>
                <li><a href="#" className="hover:text-white">Compliance</a></li>
              </ul>
            </div>
            
            <div>
              <h3 className="font-semibold mb-4">Support</h3>
              <ul className="space-y-2 text-gray-400">
                <li><a href="#" className="hover:text-white">Help Center</a></li>
                <li><a href="#" className="hover:text-white">Contact</a></li>
                <li><a href="tel:+254712826551" className="hover:text-white">+254 712 826 551</a></li>
              </ul>
            </div>
          </div>
          
          <div className="border-t border-gray-800 mt-8 pt-8 text-center text-gray-400">
            <p>&copy; 2025 PesaBit. Licensed under MIT License.</p>
          </div>
        </div>
      </footer>
    </div>
  );
}