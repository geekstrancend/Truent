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
      <head>
        {/* Loaded via <link> rather than next/font because next/font resolves
            Google Fonts at build time, which fails in network-restricted build
            environments. Matches the source design's own font loading. */}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="" />
        {/* eslint-disable-next-line @next/next/no-page-custom-font -- this is
            the root layout, so the font applies to every route, not one page.
            Swap to next/font once builds can reach fonts.googleapis.com. */}
        <link
          href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=IBM+Plex+Mono:wght@400;500;600&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>
        <AuthProvider>
          {children}
        </AuthProvider>
      </body>
    </html>
  )
}
