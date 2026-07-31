'use client'

import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-bg flex flex-col">
      <MarketingNav />

      <main className="flex-1 px-6 py-24 max-w-4xl mx-auto">
        <h1 className="font-display text-5xl font-[700] text-text mb-4">
          Terms of Service
        </h1>
        <p className="text-body-lg text-sec mb-12">
          Last updated: July 2026
        </p>

        <div className="space-y-8 prose prose-invert max-w-none">
          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              1. Agreement to Terms
            </h2>
            <p className="text-body-md text-sec leading-7">
              {'These Terms of Service ("Terms") constitute a legally binding agreement between you ("User" or "you") and Truent Security, Inc. ("Company," "we," "us," or "our"). By accessing and using the Truent platform, website, and services, you acknowledge that you have read, understood, and agree to be bound by all the terms and conditions contained herein.'}
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              2. Use License
            </h2>
            <p className="text-body-md text-sec leading-7 mb-3">
              Permission is granted to temporarily download one copy of the materials (information or software) on Truent&apos;s website for personal, non-commercial transitory viewing only. This is the grant of a license, not a transfer of title, and under this license you may not:
            </p>
            <ul className="list-disc list-inside space-y-2 text-body-md text-sec">
              <li>Modify or copy the materials</li>
              <li>Use the materials for any commercial purpose or for any public display</li>
              <li>Attempt to decompile or reverse engineer any software contained on the website</li>
              <li>Remove any copyright or other proprietary notations from the materials</li>
              <li>{'Transfer the materials to another person or "mirror" the materials on any other server'}</li>
            </ul>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              3. Disclaimer of Warranties
            </h2>
            <p className="text-body-md text-sec leading-7">
              {"The materials on Truent's website are provided on an 'as is' basis. Truent makes no warranties, expressed or implied, and hereby disclaims and negates all other warranties including, without limitation, implied warranties or conditions of merchantability, fitness for a particular purpose, or non-infringement of intellectual property or other violation of rights."}
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              4. Limitations of Liability
            </h2>
            <p className="text-body-md text-sec leading-7">
              In no event shall Truent Security or its suppliers be liable for any damages (including, without limitation, damages for loss of data or profit, or due to business interruption) arising out of the use or inability to use the materials on Truent&apos;s website, even if Truent or an authorized representative has been notified orally or in writing of the possibility of such damage.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              5. Accuracy of Materials
            </h2>
            <p className="text-body-md text-sec leading-7">
              The materials appearing on Truent&apos;s website could include technical, typographical, or photographic errors. Truent does not warrant that any of the materials on its website are accurate, complete, or current. Truent may make changes to the materials contained on its website at any time without notice.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              6. Acceptable Use Policy
            </h2>
            <p className="text-body-md text-sec leading-7 mb-3">
              You agree not to:
            </p>
            <ul className="list-disc list-inside space-y-2 text-body-md text-sec">
              <li>Use the service for illegal or unauthorized purposes</li>
              <li>Attempt to gain unauthorized access to our systems</li>
              <li>Submit malicious code or attempt to compromise system security</li>
              <li>Spam or abuse other users</li>
              <li>Violate any applicable laws or regulations</li>
            </ul>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              7. Intellectual Property Rights
            </h2>
            <p className="text-body-md text-sec leading-7">
              Unless otherwise stated, Truent owns the intellectual property rights for all material on its website. All intellectual property rights are reserved. You may access this from the website for personally educational and security research purposes, provided you do not modify any materials and provided you do not use them for any commercial purpose or any public display.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              8. Termination
            </h2>
            <p className="text-body-md text-sec leading-7">
              These Terms are effective unless and until terminated by either you or Truent. Your rights under these Terms will terminate automatically without notice from Truent if you fail to comply with any terms or conditions of these Terms.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              9. Governing Law
            </h2>
            <p className="text-body-md text-sec leading-7">
              These terms and conditions are governed by and construed in accordance with the laws of the jurisdiction where Truent is located, and you irrevocably submit to the exclusive jurisdiction of the courts in that location.
            </p>
          </section>

          <section>
            <h2 className="font-display text-2xl font-[600] text-text mb-3">
              10. Contact Information
            </h2>
            <p className="text-body-md text-sec leading-7">
              If you have any questions about these Terms, please contact us at:
            </p>
            <div className="mt-3 text-body-md text-sec">
              <p>Email: legal@truent.dev</p>
              <p>Address: Truent Security, Inc.</p>
            </div>
          </section>
        </div>
      </main>

      <MarketingFooter />
    </div>
  )
}
