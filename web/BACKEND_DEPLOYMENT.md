# Backend deployment

The web backend is implemented as Next.js route handlers backed by Prisma. It
stores users, database sessions, scans, findings, subscriptions, and payment
records. Every scan/report query is scoped to the authenticated user.

## Required production configuration

- `DATABASE_URL`: a durable SQLite volume for the current schema. Do not use an
  ephemeral serverless filesystem. A PostgreSQL migration is recommended before
  horizontally scaling the application.
- `NEXTAUTH_URL` and a randomly generated `NEXTAUTH_SECRET`.
- OAuth credentials for each enabled provider.
- `ANTHROPIC_API_KEY` for AI-assisted findings. Pattern detection still runs if
  this key is absent.
- `STRIPE_SECRET_KEY` and `STRIPE_WEBHOOK_SECRET`. Configure Stripe to deliver
  `checkout.session.completed`, `customer.subscription.updated`, and
  `customer.subscription.deleted` to `/api/payment/webhook`.

Deploy migrations before starting an application release:

```bash
npx prisma migrate deploy
npm run build
npm start
```

The crypto payment route remains fail-closed and must not be used to provision
subscriptions until invoice amount and ERC-20 transfer verification are added.
GitHub repository scans similarly return an explicit unsupported response until
the GitHub App installation/token flow is implemented; direct code and file
scans are available now.

## API surface

- `POST /api/analyze`: authenticated scan submission, durable quotas and results.
- `POST /api/auth/wallet-nonce`: one-time, five-minute wallet sign-in challenge.
- `GET /api/scans`: authenticated scan history.
- `GET /api/scans/:id`: owned report and findings.
- `PATCH /api/scans/:id`: update an owned finding's workflow status.
- `POST /api/payment/create-checkout`: server-priced Stripe checkout.
- `POST /api/payment/webhook`: signature-verified subscription provisioning.
