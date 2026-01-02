# syntax=docker/dockerfile:1

ARG NODE_VERSION=24

FROM node:${NODE_VERSION}-alpine AS base
WORKDIR /app

RUN addgroup -g 1001 -S nodejs && \
  adduser -S nodejs -u 1001 -G nodejs && \
  chown -R nodejs:nodejs /app

FROM base AS deps

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
  --mount=type=bind,source=package.json,target=package.json \
  --mount=type=bind,source=package-lock.json,target=package-lock.json \
  npm ci --omit=dev && \
  npm cache clean --force

RUN chown -R nodejs:nodejs /app

FROM base AS build-deps

RUN --mount=type=cache,target=/root/.npm,sharing=locked \
  --mount=type=bind,source=package.json,target=package.json \
  --mount=type=bind,source=package-lock.json,target=package-lock.json \
  npm ci --no-audit --no-fund && \
  npm cache clean --force

RUN chown -R nodejs:nodejs /app

FROM build-deps AS build

RUN --mount=type=bind,source=app,target=app \
  --mount=type=bind,source=next.config.js,target=next.config.js \
  --mount=type=bind,source=package.json,target=package.json \
  --mount=type=bind,source=package-lock.json,target=package-lock.json \
  --mount=type=bind,source=tsconfig.json,target=tsconfig.json \
  npm run build

RUN chown -R nodejs:nodejs /app

FROM base AS production
ARG BACKEND_HOST
ARG BACKEND_PORT

ENV NODE_ENV=production \
  NODE_OPTIONS="--max-old-space-size=256 --no-warnings" \
  NPM_CONFIG_LOGLEVEL=silent \
  BACKEND_HOST=${BACKEND_HOST} \
  BACKEND_PORT=${BACKEND_PORT}

COPY --from=build --chown=nodejs:nodejs /app/.next ./.next
COPY --from=deps --chown=nodejs:nodejs /app/node_modules ./node_modules
COPY --chown=nodejs:nodejs package*.json ./

USER nodejs

EXPOSE 3000

CMD ["npm", "run", "start"]
