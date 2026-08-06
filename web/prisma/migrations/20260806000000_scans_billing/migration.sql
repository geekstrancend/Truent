-- CreateTable
CREATE TABLE "Scan" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "projectName" TEXT,
    "sourceType" TEXT NOT NULL DEFAULT 'code',
    "sourceRef" TEXT,
    "language" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'processing',
    "error" TEXT,
    "startedAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "completedAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "Scan_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "Finding" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "scanId" TEXT NOT NULL,
    "severity" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "location" TEXT,
    "line" INTEGER,
    "impact" TEXT,
    "recommendation" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'open',
    "fingerprint" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "Finding_scanId_fkey" FOREIGN KEY ("scanId") REFERENCES "Scan" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "Subscription" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "provider" TEXT NOT NULL,
    "providerCustomerId" TEXT,
    "providerSubscriptionId" TEXT,
    "plan" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "currentPeriodEnd" DATETIME,
    "cancelAtPeriodEnd" BOOLEAN NOT NULL DEFAULT false,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "Subscription_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "Payment" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "userId" TEXT NOT NULL,
    "provider" TEXT NOT NULL,
    "providerPaymentId" TEXT NOT NULL,
    "plan" TEXT NOT NULL,
    "amount" INTEGER NOT NULL,
    "currency" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "walletAddress" TEXT,
    "transactionHash" TEXT,
    "expiresAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "Payment_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE "AuthNonce" (
    "address" TEXT NOT NULL PRIMARY KEY,
    "nonceHash" TEXT NOT NULL,
    "expiresAt" DATETIME NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX "Scan_userId_createdAt_idx" ON "Scan"("userId", "createdAt");
CREATE INDEX "Scan_status_idx" ON "Scan"("status");
CREATE UNIQUE INDEX "Finding_scanId_fingerprint_key" ON "Finding"("scanId", "fingerprint");
CREATE INDEX "Finding_scanId_severity_idx" ON "Finding"("scanId", "severity");
CREATE UNIQUE INDEX "Subscription_userId_key" ON "Subscription"("userId");
CREATE UNIQUE INDEX "Subscription_providerSubscriptionId_key" ON "Subscription"("providerSubscriptionId");
CREATE INDEX "Subscription_status_idx" ON "Subscription"("status");
CREATE UNIQUE INDEX "Payment_providerPaymentId_key" ON "Payment"("providerPaymentId");
CREATE UNIQUE INDEX "Payment_transactionHash_key" ON "Payment"("transactionHash");
CREATE INDEX "Payment_userId_createdAt_idx" ON "Payment"("userId", "createdAt");
CREATE INDEX "Payment_status_idx" ON "Payment"("status");
CREATE INDEX "AuthNonce_expiresAt_idx" ON "AuthNonce"("expiresAt");
