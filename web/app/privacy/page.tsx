import type { Metadata } from 'next'
import { LegalPage, type LegalSection } from '@/components/layout/LegalPage'

export const metadata: Metadata = {
  title: 'Privacy Policy — Truent',
  description: 'How Truent Security collects, uses, and safeguards your information.',
}

const sections: LegalSection[] = [
  {
    num: '01',
    title: 'Introduction',
    body: 'Truent Security ("we," "us," "our," or "Company") is committed to protecting your privacy. This Privacy Policy explains how we collect, use, disclose, and safeguard your information when you use our website and services.',
  },
  {
    num: '02',
    title: 'Information we collect',
    body: 'We may collect information about you in a variety of ways. The information we may collect on our site includes:',
    items: [
      'Personal data: email address, name, phone number, and other contact information you provide',
      'Smart contract data: code submitted for analysis, stored securely and used only for scanning',
      'Usage data: pages visited, time spent on site, and other analytics information',
      'Device data: IP address, browser type, and device information',
    ],
  },
  {
    num: '03',
    title: 'Use of your information',
    body: 'Having accurate information about you permits us to provide you with a smooth, efficient, and customized experience. Specifically, we may use information collected about you to:',
    items: [
      'Provide and maintain our services',
      'Send administrative information and security alerts',
      'Respond to your inquiries and customer service requests',
      'Improve our website and services',
      'Analyze usage patterns to enhance user experience',
    ],
  },
  {
    num: '04',
    title: 'Disclosure of your information',
    body: 'We may share information we have collected about you in certain situations:',
    items: [
      'By law or to protect rights: if required by law or to protect our rights, privacy, safety, or property',
      'Service providers: we may share information with third parties who provide services on our behalf',
      'Business transfers: your information may be transferred as part of a merger, acquisition, or bankruptcy',
    ],
  },
  {
    num: '05',
    title: 'Security of your information',
    body: 'We use administrative, technical, and physical security measures to protect your personal information. Submitted contract code is encrypted at rest and never used to train models. However, no method of transmission over the internet is 100% secure, and we cannot guarantee absolute security.',
  },
]

export default function PrivacyPage() {
  return (
    <LegalPage
      title="Privacy Policy"
      updated="JULY 2026"
      sections={sections}
      contact={{
        num: '06',
        title: 'Contact us',
        body: 'If you have questions or comments about this Privacy Policy, please contact us at:',
        email: 'privacy@truent.dev',
      }}
      omitFooterLink="Privacy"
    />
  )
}
