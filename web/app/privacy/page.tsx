'use client'

import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-bg flex flex-col">
      <MarketingNav />

      <main className="flex-1 px-6 py-24 max-w-4xl mx-auto">
        <h1 className="font-display text-5xl font-[700] text-text mb-4">
          Privacy Policy
        </h1>
        <p className="text-body-lg text-sec mb-12">
          Last updated: July 2026
        </p>

        <div className="space-y-8 prose prose-invert max-w-none">
          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              1. Introduction
            </h2>
            <p className="text-body-md text-sec leading-7">
              {'Truent Security ("we," "us," "our," or "Company") is committed to protecting your privacy. This Privacy Policy explains how we collect, use, disclose, and safeguard your information when you use our website and services.'}
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              2. Information We Collect
            </h2>
            <p className="text-body-md text-sec leading-7 mb-3">
              We may collect information about you in a variety of ways. The information we may collect on our site includes:
            </p>
            <ul className="list-disc list-inside space-y-2 text-body-md text-sec">
              <li>Personal Data: Email address, name, phone number, and other contact information you provide</li>
              <li>Smart Contract Data: Code submitted for analysis (stored securely and used only for scanning)</li>
              <li>Usage Data: Pages visited, time spent on site, and other analytics information</li>
              <li>Device Data: IP address, browser type, and device information</li>
            </ul>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              3. Use of Your Information
            </h2>
            <p className="text-body-md text-sec leading-7 mb-3">
              Having accurate information about you permits us to provide you with a smooth, efficient, and customized experience. Specifically, we may use information collected about you via the site to:
            </p>
            <ul className="list-disc list-inside space-y-2 text-body-md text-sec">
              <li>Provide and maintain our services</li>
              <li>Send administrative information and security alerts</li>
              <li>Respond to your inquiries and customer service requests</li>
              <li>Improve our website and services</li>
              <li>Analyze usage patterns to enhance user experience</li>
            </ul>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              4. Disclosure of Your Information
            </h2>
            <p className="text-body-md text-sec leading-7">
              We may share information we have collected about you in certain situations:
            </p>
            <ul className="list-disc list-inside space-y-2 text-body-md text-sec mt-3">
              <li><strong>By Law or to Protect Rights:</strong> If required by law or to protect our rights, privacy, safety, or property</li>
              <li><strong>Service Providers:</strong> We may share your information with third parties who provide services on our behalf</li>
              <li><strong>Business Transfers:</strong> Your information may be transferred as part of a merger, acquisition, or bankruptcy</li>
            </ul>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              5. Security of Your Information
            </h2>
            <p className="text-body-md text-sec leading-7">
              We use administrative, technical, and physical security measures to protect your personal information. However, no method of transmission over the internet is 100% secure. We cannot guarantee absolute security.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              6. Contact Us
            </h2>
            <p className="text-body-md text-sec leading-7">
              If you have questions or comments about this Privacy Policy, please contact us at:
            </p>
            <div className="mt-3 text-body-md text-sec">
              <p>Email: privacy@truent.dev</p>
              <p>Address: Truent Security, Inc.</p>
            </div>
          </section>
        </div>
      </main>

      <MarketingFooter />
    </div>
  )
}
