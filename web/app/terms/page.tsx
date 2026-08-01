import type { Metadata } from 'next'
import { LegalPage, type LegalSection } from '@/components/layout/LegalPage'

export const metadata: Metadata = {
  title: 'Terms of Service — Truent',
  description: 'The terms governing your use of the Truent platform and services.',
}

const sections: LegalSection[] = [
  {
    num: '01',
    title: 'Agreement to terms',
    body: 'These Terms of Service ("Terms") constitute a legally binding agreement between you ("User" or "you") and Truent Security, Inc. ("Company," "we," "us," or "our"). By accessing and using the Truent platform, website, and services, you acknowledge that you have read, understood, and agree to be bound by all the terms and conditions contained herein.',
  },
  {
    num: '02',
    title: 'Use license',
    body: 'Permission is granted to temporarily download one copy of the materials on Truent\'s website for personal, non-commercial transitory viewing only. This is the grant of a license, not a transfer of title, and under this license you may not:',
    items: [
      'Modify or copy the materials',
      'Use the materials for any commercial purpose or public display',
      'Attempt to decompile or reverse engineer any software contained on the website',
      'Remove any copyright or other proprietary notations from the materials',
      'Transfer the materials to another person or mirror them on any other server',
    ],
  },
  {
    num: '03',
    title: 'Disclaimer of warranties',
    body: 'The materials on Truent\'s website are provided on an "as is" basis. Truent makes no warranties, expressed or implied, and hereby disclaims and negates all other warranties including, without limitation, implied warranties or conditions of merchantability, fitness for a particular purpose, or non-infringement of intellectual property or other violation of rights.',
  },
  {
    num: '04',
    title: 'Limitations of liability',
    body: 'In no event shall Truent Security or its suppliers be liable for any damages (including, without limitation, damages for loss of data or profit, or due to business interruption) arising out of the use or inability to use the materials on Truent\'s website, even if Truent or an authorized representative has been notified orally or in writing of the possibility of such damage.',
  },
  {
    num: '05',
    title: 'Accuracy of materials',
    body: 'The materials appearing on Truent\'s website could include technical, typographical, or photographic errors. Truent does not warrant that any of the materials on its website are accurate, complete, or current. Truent may make changes to the materials contained on its website at any time without notice.',
  },
  {
    num: '06',
    title: 'Acceptable use policy',
    body: 'You agree not to:',
    items: [
      'Use the service for illegal or unauthorized purposes',
      'Attempt to gain unauthorized access to our systems',
      'Submit malicious code or attempt to compromise system security',
      'Spam or abuse other users',
      'Violate any applicable laws or regulations',
    ],
  },
  {
    num: '07',
    title: 'Intellectual property rights',
    body: 'Unless otherwise stated, Truent owns the intellectual property rights for all material on its website. All intellectual property rights are reserved. You may access this from the website for personal educational and security research purposes, provided you do not modify any materials and do not use them for any commercial purpose or public display.',
  },
  {
    num: '08',
    title: 'Termination',
    body: 'These Terms are effective unless and until terminated by either you or Truent. Your rights under these Terms will terminate automatically without notice from Truent if you fail to comply with any terms or conditions of these Terms.',
  },
  {
    num: '09',
    title: 'Governing law',
    body: 'These terms and conditions are governed by and construed in accordance with the laws of the jurisdiction where Truent is located, and you irrevocably submit to the exclusive jurisdiction of the courts in that location.',
  },
]

export default function TermsPage() {
  return (
    <LegalPage
      title="Terms of Service"
      updated="JULY 2026"
      sections={sections}
      contact={{
        num: '10',
        title: 'Contact information',
        body: 'If you have any questions about these Terms, please contact us at:',
        email: 'legal@truent.dev',
      }}
      omitFooterLink="Terms"
    />
  )
}
