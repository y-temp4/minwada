FROM node:22.11.0-alpine3.20 AS base

FROM base AS builder

WORKDIR /app

ENV NEXT_TELEMETRY_DISABLED 1

ARG API_METRICS_TOKEN \
    NEXT_PUBLIC_API_BASE_URL \
    NEXT_PUBLIC_API_TIMEOUT \
    NEXT_PUBLIC_APP_NAME \
    NEXT_PUBLIC_APP_URL

COPY . .

RUN npm ci
RUN npm run build

FROM base AS runner

WORKDIR /app

ENV NODE_ENV production
ENV NEXT_TELEMETRY_DISABLED 1

RUN addgroup --system --gid 1001 nodejs
RUN adduser --system --uid 1001 nextjs

RUN mkdir .next
RUN chown nextjs:nodejs .next

COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

ENV PORT 3000
ENV HOSTNAME "0.0.0.0"

CMD ["node", "server.js"]
