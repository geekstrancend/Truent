import Link from 'next/link'
import { ShieldCheck, Github, Twitter, ExternalLink } from 'lucide-react'

export function MarketingFooter() {
  const currentYear = new Date().getFullYear()

  return (
    <footer className="border-t border-hair bg-surface-2">
      <div className="max-w-site mx-auto px-7 pt-16 pb-8">

        {/* Top grid */}
        <div className="grid grid-cols-2 md:grid-cols-5 gap-8 mb-16">
          {/* Brand column */}
          <div className="col-span-2 md:col-span-2 pr-8">
            <Link href="/" className="flex items-center gap-2 mb-4">
              <ShieldCheck size={20} className="text-acc-text" />
              <span className="font-mono font-[600] text-text text-base">Truent</span>
            </Link>
            <p className="text-sec text-body-md mb-6 leading-6">
              The invariant-driven smart contract security platform. Don&apos;t get Hacked.
            </p>
            <div className="flex items-center gap-3">
              <a
                href="https://github.com/geekstrancend/Truent"
                target="_blank"
                rel="noopener noreferrer"
                className="w-9 h-9 flex items-center justify-center rounded-lg bg-panel border border-hair text-sec hover:text-text hover:border-indigo transition-colors"
                aria-label="GitHub"
              >
                <Github size={16} />
              </a>
              <a
                href="https://twitter.com/truentsec"
                target="_blank"
                rel="noopener noreferrer"
                className="w-9 h-9 flex items-center justify-center rounded-lg bg-panel border border-hair text-sec hover:text-text hover:border-indigo transition-colors"
                aria-label="Twitter"
              >
                <Twitter size={16} />
              </a>
            </div>
          </div>

          {/* Product */}
          <div>
            <h3 className="text-label-sm text-text mb-5">Product</h3>
            <ul className="space-y-3">
              {[
                { label: 'Features', href: '/#features' },
                { label: 'Pricing', href: '/pricing' },
                { label: 'Security Library', href: '/library' },
                { label: 'Changelog', href: '/docs' },
              ].map((link) => (
                <li key={link.href}>
                  <Link href={link.href} className="block text-sec hover:text-text text-body-md transition-colors">
                    {link.label}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          {/* Resources */}
          <div>
            <h3 className="text-label-sm text-text mb-5">Resources</h3>
            <ul className="space-y-3">
              {[
                { label: 'Documentation', href: '/docs' },
                { label: 'Getting Started', href: '/docs/getting-started' },
                { label: 'CLI Reference', href: '/docs/cli' },
                { label: 'CI/CD Guide', href: '/docs/ci-cd' },
                { label: 'GitHub', href: 'https://github.com/geekstrancend/Truent', external: true },
              ].map((link) => (
                <li key={link.href}>
                  <Link
                    href={link.href}
                    target={link.external ? '_blank' : undefined}
                    rel={link.external ? 'noopener noreferrer' : undefined}
                    className="inline-flex items-center gap-1 text-sec hover:text-text text-body-md transition-colors"
                  >
                    {link.label}
                    {link.external && <ExternalLink size={10} className="opacity-60" />}
                  </Link>
                </li>
              ))}
            </ul>
          </div>

          {/* Legal */}
          <div>
            <h3 className="text-label-sm text-text mb-5">Company</h3>
            <ul className="space-y-3">
              {[
                { label: 'Contact Sales', href: '/contact' },
                { label: 'Privacy Policy', href: '/privacy' },
                { label: 'Terms of Service', href: '/terms' },
                { label: 'Security Disclosure', href: 'https://github.com/geekstrancend/Truent/security/policy', external: true },
              ].map((link) => (
                <li key={link.href}>
                  <Link
                    href={link.href}
                    target={link.external ? '_blank' : undefined}
                    rel={link.external ? 'noopener noreferrer' : undefined}
                    className="inline-flex items-center gap-1 text-sec hover:text-text text-body-md transition-colors"
                  >
                    {link.label}
                    {link.external && <ExternalLink size={10} className="opacity-60" />}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>

        {/* Divider */}
        <div className="section-divider mb-8" />

        {/* Bottom bar */}
        <div className="flex flex-col md:flex-row items-center justify-between gap-4">
          <p className="text-sec text-xs">
            © {currentYear} Truent Security, Inc. All rights reserved.
          </p>
          <div className="flex items-center gap-6 text-xs text-sec">
            <Link href="/privacy" className="hover:text-text transition-colors">Privacy</Link>
            <Link href="/terms" className="hover:text-text transition-colors">Terms</Link>
            <a href="mailto:contact@truent.dev" className="hover:text-text transition-colors">contact@truent.dev</a>
          </div>
        </div>
      </div>
    </footer>
  )
}

