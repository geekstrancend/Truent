# Truent Web Authentication Setup

This guide explains how to set up and use the authentication system in the Truent web application.

## Overview

The authentication system uses:
- **Next Auth 4.24.14** for session management
- **Prisma 7** for database models, via `@prisma/adapter-better-sqlite3` (Prisma 7 requires
  an explicit driver adapter - `new PrismaClient()` with no arguments no longer works)
- **SQLite** for data storage by default (`prisma/dev.db`); `@prisma/adapter-pg` is installed
  for a future Postgres migration but not wired up - switching requires changing
  `schema.prisma`'s provider and regenerating migrations, not just swapping the adapter
- **Bcrypt** for password hashing, with a constant-time comparison against a dummy hash on
  an unknown email so response timing can't be used to enumerate accounts
- Four providers: **Credentials** (email/password), **GitHub**, **Google**, and **Web3
  Wallet** (signature-based)

All provider/session config lives in `web/lib/auth-options.ts`, imported by both the
NextAuth route handler and any API route that needs the current session server-side.

## Setup Instructions

### 1. Generate NEXTAUTH_SECRET

Generate a secure secret for NextAuth:

```bash
openssl rand -base64 32
```

Copy this value to your `.env.local` file.

### 2. Configure Database

Set your `DATABASE_URL` in `.env.local` (defaults to a local SQLite file if unset):

```bash
DATABASE_URL="file:./prisma/dev.db"
```

### 3. Run Prisma Migrations

```bash
cd web
npx prisma migrate dev
```

This creates the necessary tables:
- `User` - User account information
- `Account` - OAuth account links
- `Session` - User sessions
- `VerificationToken` - For email verification (optional)

### 4. GitHub OAuth Setup (Optional)

To enable GitHub sign-in:

1. Go to GitHub Settings → Developer settings → OAuth Apps
2. Create a new OAuth App:
   - **Application name**: Truent
   - **Homepage URL**: http://localhost:3000 (development)
   - **Authorization callback URL**: http://localhost:3000/api/auth/callback/github
3. Copy the **Client ID** and **Client Secret**
4. Add to `.env.local`:
   ```
   GITHUB_ID="your-client-id"
   GITHUB_SECRET="your-client-secret"
   ```

### 5. Required Environment Variables

Copy `.env.example` to `.env.local` and fill in the values:

```bash
cp .env.example .env.local
```

## Usage

### For Users

Users can sign in using:

1. **Email/Password** - Create an account with email and password
2. **GitHub** - Quick sign-in with GitHub account

### For Developers

#### Check Authentication Status

```tsx
'use client'

import { useSession } from 'next-auth/react'

export function MyComponent() {
  const { data: session, status } = useSession()

  if (status === 'loading') return <div>Loading...</div>
  if (status === 'unauthenticated') return <div>Not signed in</div>

  return <div>Welcome {session?.user?.name}</div>
}
```

#### Sign Out

```tsx
import { signOut } from 'next-auth/react'

<button onClick={() => signOut()}>Sign Out</button>
```

#### Protect Routes

Routes under `/dashboard/*` and `/reports/*` are automatically protected by the middleware in `middleware.ts`.

## API Endpoints

### `POST /api/auth/signup`

Create a new user account.

**Request:**
```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "securepassword"
}
```

**Response (201):**
```json
{
  "id": "user-id",
  "name": "John Doe",
  "email": "john@example.com",
  "createdAt": "2024-01-01T00:00:00Z"
}
```

### `POST /api/auth/callback/credentials`

Sign in with email and password (handled by Next Auth).

### `GET /api/auth/session`

Get current user session (protected route).

## Database Schema

### User Model

```
- id: String (primary key)
- name: String?
- email: String (unique)
- emailVerified: DateTime?
- password: String? (for credentials provider)
- image: String?
- createdAt: DateTime
- updatedAt: DateTime
- accounts: Account[] (OAuth accounts)
- sessions: Session[] (Active sessions)
```

### Account Model

```
- id: String (primary key)
- userId: String (foreign key)
- type: String
- provider: String
- providerAccountId: String
- refresh_token: String?
- access_token: String?
- expires_at: Int?
- token_type: String?
- scope: String?
- id_token: String?
- session_state: String?
```

### Session Model

```
- id: String (primary key)
- sessionToken: String (unique)
- userId: String (foreign key)
- expires: DateTime
```

## Security Considerations

1. **Passwords**: Always hashed with bcrypt before storage
2. **Secrets**: Store NEXTAUTH_SECRET securely (never commit to git)
3. **HTTPS**: Use HTTPS in production
4. **Session Timeout**: Sessions expire after 7 days
5. **CSRF Protection**: Built into Next Auth

## Troubleshooting

### "Missing credentials" error

Ensure `NEXTAUTH_URL` and `NEXTAUTH_SECRET` are set correctly in `.env.local`.

### OAuth callback fails

Check that the callback URL matches exactly in your OAuth provider settings:
- Development: `http://localhost:3000/api/auth/callback/[provider]`
- Production: `https://yourdomain.com/api/auth/callback/[provider]`

### Database connection error

Verify `DATABASE_URL` is correct and the database is running.

## Not Yet Implemented

1. Email verification
2. "Forgot Password" flow
3. Additional OAuth providers (Microsoft)
4. User profile management page
5. Role-based access control (RBAC)

## Resources

- [Next Auth Documentation](https://next-auth.js.org/)
- [Prisma Documentation](https://www.prisma.io/docs/)
- [GitHub OAuth Documentation](https://docs.github.com/en/developers/apps/building-oauth-apps)
