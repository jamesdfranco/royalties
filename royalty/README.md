# Royalties.fun

A minimalist, professional on-chain royalty marketplace for internet creators. Built on Solana.

## Overview

Royalties.fun is a marketplace where creators can sell defined portions of their future revenue (royalties) to buyers in exchange for upfront capital.

### For Creators
- Musicians selling a percentage of streaming revenue
- YouTubers selling a % of AdSense from a video or channel
- Streamers selling a % of subscription revenue
- Writers selling a % of book or digital sales
- Developers selling % of an indie game's earnings

### For Investors
- Buy royalty tokens representing legal ownership of creator revenue
- Trade tokens on the secondary market
- Earn passive income from creator success

## Features

- **Primary Market**: Browse and purchase royalty listings directly from creators
- **Secondary Market**: Trade royalty tokens with other investors
- **Creator Dashboard**: Sell royalties and manage your listings
- **Investor Dashboard**: Track your tokens, earnings, and claim payouts
- **NFT Receipts**: Each purchase mints an NFT linking to the legal contract

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **Blockchain**: Solana
- **Wallet**: @solana/wallet-adapter-react

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Run development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to view the application.

## Project Structure

```
/app
  layout.tsx          # Root layout with Navbar, Footer, WalletProvider
  page.tsx            # Homepage
  /marketplace        # Primary market listings
  /sell               # Creator royalty listing form
  /secondary          # Secondary market for token resales
  /dashboard          # User dashboard for tokens and payouts

/components
  Navbar.tsx          # Sticky navigation
  Footer.tsx          # Site footer
  Hero.tsx            # Homepage hero section
  Card.tsx            # Generic sharp-edged card
  ListingCard.tsx     # Royalty listing display
  SectionHeader.tsx   # Section title component
  PlaceholderCrownLogo.tsx  # Brand logo
  WalletProvider.tsx  # Solana wallet context

/public
  logo.svg            # Crown logo asset
```

## Design System

- Pure white background (#FFFFFF)
- Black text (#000000)
- Sharp edges everywhere (no rounded corners)
- Large bold typography
- Generous whitespace
- Financial/contractual tone

## Current Status

This is the MVP frontend with placeholder data. Backend logic, smart contracts, and API integrations are planned for future versions.

## License

MIT
