'use client'

import { useState } from 'react'
import { signIn } from 'next-auth/react'
import { X, Eye, EyeOff, Mail, Lock, User, Wallet } from 'lucide-react'
import { Button } from './Button'
import { useEscapeKey } from '@/components/hooks/useEscapeKey'

interface AuthModalProps {
  isOpen: boolean
  onClose: () => void
  defaultTab?: 'signin' | 'signup'
}

export function AuthModal({ isOpen, onClose, defaultTab = 'signin' }: AuthModalProps) {
  const [tab, setTab] = useState<'signin' | 'signup'>(defaultTab)
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [fullName, setFullName] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEscapeKey(isOpen, onClose)

  if (!isOpen) return null

  const handleSignIn = async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await signIn('credentials', {
        email,
        password,
        redirect: true,
        callbackUrl: '/dashboard',
      })
      
      if (!result?.ok) {
        setError('Invalid email or password')
      }
    } catch (err) {
      setError('An error occurred. Please try again.')
      console.error('Sign in error:', err)
    } finally {
      setIsLoading(false)
    }
  }

  const handleSignUp = async () => {
    setIsLoading(true)
    setError(null)
    try {
      const response = await fetch('/api/auth/signup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: fullName,
          email,
          password,
        }),
      })

      if (!response.ok) {
        const data = await response.json()
        setError(data.error || 'Sign up failed')
        return
      }

      // Sign in after successful signup
      const signInResult = await signIn('credentials', {
        email,
        password,
        redirect: true,
        callbackUrl: '/dashboard',
      })

      if (!signInResult?.ok) {
        setError('Account created but sign in failed. Please try signing in.')
      }
    } catch (err) {
      setError('An error occurred. Please try again.')
      console.error('Sign up error:', err)
    } finally {
      setIsLoading(false)
    }
  }

  const handleOAuthSignIn = (provider: 'github' | 'google') => {
    signIn(provider, { callbackUrl: '/dashboard' })
  }

  const handleWalletConnect = async () => {
    setIsLoading(true)
    setError(null)
    try {
      // Request wallet connection (MetaMask, WalletConnect, etc.)
      if (!window.ethereum) {
        setError('Web3 wallet not detected. Please install MetaMask or use WalletConnect.')
        setIsLoading(false)
        return
      }

      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts',
      })

      if (!accounts || accounts.length === 0) {
        setError('Wallet connection denied')
        setIsLoading(false)
        return
      }

      const address = accounts[0]
      const message = `Sign this message to authenticate with Truent. Timestamp: ${Date.now()}`

      // Request signature
      const signature = await window.ethereum.request({
        method: 'personal_sign',
        params: [message, address],
      })

      // Sign in with wallet
      const result = await signIn('wallet', {
        address,
        message,
        signature,
        redirect: true,
        callbackUrl: '/dashboard',
      })

      if (!result?.ok) {
        setError('Wallet authentication failed')
      }
    } catch (err) {
      setError('Wallet connection error. Please try again.')
      console.error('Wallet error:', err)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="auth-modal-title"
        className="bg-bg rounded-card shadow-2xl w-full max-w-md relative"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Close button */}
        <button
          onClick={onClose}
          aria-label="Close dialog"
          className="absolute top-3 right-3 p-2 hover:bg-panel rounded-lg transition"
        >
          <X className="w-5 h-5 text-text" />
        </button>

        {/* Logo / Header */}
        <div className="px-6 pt-6 pb-2">
          <h2 id="auth-modal-title" className="text-2xl font-bold text-text mb-1">Truent</h2>
          <p className="text-sm text-sec">Smart contract invariant checking</p>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-hair px-6">
          <button
            onClick={() => setTab('signin')}
            className={`flex-1 py-4 text-sm font-medium transition ${
              tab === 'signin'
                ? 'text-acc-text border-b-2 border-indigo'
                : 'text-sec hover:text-text'
            }`}
          >
            Sign In
          </button>
          <button
            onClick={() => setTab('signup')}
            className={`flex-1 py-4 text-sm font-medium transition ${
              tab === 'signup'
                ? 'text-acc-text border-b-2 border-indigo'
                : 'text-sec hover:text-text'
            }`}
          >
            Sign Up
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-6 space-y-4">
          {/* OAuth Buttons */}
          <div className="space-y-2">
            <p className="text-xs text-sec text-center mb-3">
              Quick sign in
            </p>
            <Button
              variant="secondary"
              className="w-full justify-center gap-2"
              onClick={() => handleOAuthSignIn('github')}
              disabled={isLoading}
            >
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v 3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
              </svg>
              GitHub
            </Button>
            <Button
              variant="secondary"
              className="w-full justify-center gap-2"
              onClick={() => handleOAuthSignIn('google')}
              disabled={isLoading}
            >
              <svg className="w-4 h-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" />
                <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" />
                <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" />
                <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" />
              </svg>
              Google
            </Button>
            <Button
              variant="secondary"
              className="w-full justify-center gap-2"
              onClick={handleWalletConnect}
              disabled={isLoading}
            >
              <Wallet className="w-4 h-4" />
              Web3 Wallet
            </Button>
          </div>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-hair"></div>
            </div>
            <div className="relative flex justify-center text-sm">
              <span className="px-2 bg-bg text-sec">or</span>
            </div>
          </div>

          {/* Email / Password Form */}
          <div className="space-y-4">
            {tab === 'signup' && (
              <div>
                <label htmlFor="auth-fullname" className="block text-sm font-medium text-text mb-2">
                  Full Name
                </label>
                <div className="relative">
                  <User className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-sec" />
                  <input
                    id="auth-fullname"
                    type="text"
                    value={fullName}
                    onChange={(e) => setFullName(e.target.value)}
                    placeholder="John Doe"
                    className="w-full pl-10 pr-4 py-2.5 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-indigo transition"
                  />
                </div>
              </div>
            )}

            <div>
              <label htmlFor="auth-email" className="block text-sm font-medium text-text mb-2">
                Email Address
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-sec" />
                <input
                  id="auth-email"
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="you@example.com"
                  className="w-full pl-10 pr-4 py-2.5 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-indigo transition"
                />
              </div>
            </div>

            <div>
              <label htmlFor="auth-password" className="block text-sm font-medium text-text mb-2">
                Password
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-sec" />
                <input
                  id="auth-password"
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  className="w-full pl-10 pr-10 py-2.5 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-indigo transition"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  aria-label={showPassword ? 'Hide password' : 'Show password'}
                  aria-pressed={showPassword}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-sec hover:text-text transition"
                >
                  {showPassword ? (
                    <EyeOff className="w-4 h-4" />
                  ) : (
                    <Eye className="w-4 h-4" />
                  )}
                </button>
              </div>
            </div>
          </div>

          {/* Forgot Password / Links */}
          {tab === 'signin' && (
            <div className="text-right">
              <button className="text-sm text-acc-text hover:text-text transition">
                Forgot password?
              </button>
            </div>
          )}

          {/* Error Message */}
          {error && (
            <div className="bg-critical-bg border border-critical-border text-critical text-sm px-3 py-2 rounded-lg">
              {error}
            </div>
          )}

          {/* CTA Button */}
          <Button
            className="w-full"
            onClick={tab === 'signin' ? handleSignIn : handleSignUp}
            disabled={isLoading}
          >
            {isLoading 
              ? (tab === 'signin' ? 'Signing in...' : 'Creating account...')
              : (tab === 'signin' ? 'Sign In' : 'Create Account')
            }
          </Button>

          {/* Trial Disclaimer */}
          <p className="text-xs text-sec text-center">
            {tab === 'signup' ? (
              <>
                Start your free 14-day trial.
                <br />
                No credit card required.
              </>
            ) : (
              <>
                Don&apos;t have an account?{' '}
                <button
                  onClick={() => setTab('signup')}
                  className="text-acc-text hover:text-text transition"
                >
                  Sign up
                </button>
              </>
            )}
          </p>
        </div>
      </div>
    </div>
  )
}
