import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Brivas - Enterprise Messaging Platform',
  description: 'The unified messaging platform for SMS, WhatsApp, USSD, and 16+ channels',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
