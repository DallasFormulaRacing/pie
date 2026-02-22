FROM node:lts AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM caddy:alpine
COPY --from=builder /app/dist /srv
COPY Caddyfile /etc/caddy/Caddyfile