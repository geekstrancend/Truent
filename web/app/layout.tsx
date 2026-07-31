import type { Metadata } from 'next'
import '../styles/globals.css'
import { AuthProvider } from './providers'

export const metadata: Metadata = {
  title: 'Truent | Smart Contract Security Intelligence',
  description: 'Audit faster. Find more. Miss nothing. Advanced symbolic execution and invariant-based security for DeFi protocols.',
  keywords: ['smart contracts', 'security', 'audit', 'DeFi', 'blockchain', 'invariants'],
  authors: [{ name: 'Truent Security' }],
  openGraph: {
    title: 'Truent | Smart Contract Security Intelligence',
    description: 'Don\'t get Hacked!',
    type: 'website',
    url: 'https://truent.dev',
  },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html
      lang="en"
      className="dark"
      suppressHydrationWarning
    >
      <body>
        <AuthProvider>
          {children}
        </AuthProvider>
      </body>
    </html>
  )
}
