/** @type {import('next').NextConfig} */
const nextConfig = {
  // The docs are one hash-routed page now, so the former sub-routes redirect
  // rather than 404 for anyone holding an old link.
  async redirects() {
    return [
      { source: '/docs/getting-started', destination: '/docs#getting-started', permanent: true },
      { source: '/docs/cli', destination: '/docs#cli', permanent: true },
      { source: '/docs/ai', destination: '/docs#ai', permanent: true },
      { source: '/docs/api', destination: '/docs#api', permanent: true },
      { source: '/docs/ci-cd', destination: '/docs#ci-cd', permanent: true },
      { source: '/docs/reports', destination: '/docs#reports', permanent: true },
    ]
  },

  reactStrictMode: true,
}

module.exports = nextConfig
