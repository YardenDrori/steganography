# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A steganography web app built as a school project. Users can hide data inside images/video. The backend is a Rust microservices architecture; the frontend is React/TypeScript.

## Repository Structure

```
src/
  backend/   # Cargo workspace with all Rust services
  frontend/  # Vite + React + TypeScript
```

## Backend Commands (run from `src/backend/`)

```bash
# Build all services
cargo build

# Build a specific service
cargo build -p auth_service

# Run a specific service locally (requires .env)
RUST_LOG=debug cargo run -p eureka_service

# Check for errors without building
cargo check

# Run tests for a specific service
cargo test -p auth_service

# Run a specific test
cargo test -p auth_service test_name

# Start all services via Docker
docker compose up --build

# Start a single service (and its dependencies)
docker compose up --build eureka_service

# Lint
cargo clippy

# After changing sqlx queries in a service (compile-time query checking)
cargo sqlx prepare -p auth_service

# Prepare DBs and run migrations locally (starts Postgres containers, migrates, then tears down)
bash prep_dbs.sh

# Smoke-test all services (must be running)
bash test_apis.sh
```

## Frontend Commands (run from `src/frontend/`)

```bash
npm run dev      # Start dev server (port 5173)
npm run build    # TypeScript check + Vite build
npm run lint     # ESLint
```

## Architecture

### Service Discovery (Eureka pattern)
`eureka_service` (port 3005) is a custom service registry — **not** Netflix Eureka. It holds:
- JWT keys (base64-encoded PEM in env vars, decoded on delivery)
- A `HashMap<service_name, ServiceEntry>` (URL + last heartbeat time)
- Token durations

Every other service **on startup**:
1. Calls `GET /config/{service_name}` on Eureka to get JWT keys and peer URLs
2. Calls `POST /register` to announce itself
3. Spawns two background tasks: heartbeat every 30s (`POST /heartbeat`), config refresh every 30s

Eureka prunes services not seen in 90s (checked every 60s). Only `auth_service` receives the JWT private key and token durations; all others get only the public key.

### API Gateway (port 3000)
Thin reverse proxy. Strips the `/api` prefix and forwards:
- `/api/auth/*` → `auth_service`
- `/api/users/*` → `user_service`
- `/api/files/*` → `files_service`
- `/api/embed/*` → `steganography_service`

Service URLs are resolved at request time from the in-memory `EurekaConfig`. CORS is configured for the frontend origin (defaults to `http://localhost:5173`).

### Services

| Service | Port | DB | Notes |
|---|---|---|---|
| `eureka_service` | 3005 | none | Must start first |
| `api_gateway` | 3000 | none | Depends on eureka |
| `auth_service` | 3001 | Postgres (port 5433) | Issues JWTs, manages refresh tokens |
| `user_service` | 3002 | Postgres (port 5434) | User CRUD; auth_service depends on it |
| `steganography_service` | 3003 | none | Image steganography logic |
| `files_service` | 3004 | Postgres (port 5435) | Currently a skeleton, no routes yet |

### Shared Crates (`src/backend/shared/`)
- `shared/global` — Eureka client functions, JWT verification (`RS256`), Axum extractors, Postgres pool helper, error types
  - `auth::user_extractors` — `AuthenticatedUser`, `RequireAdmin` (for user-facing routes)
  - `auth::service_extractors` — `InternalService` extractor (for service-to-service routes, validates a shared internal token)
  - `auth::hybrid_extractors` — routes that accept either
- `shared/auth_user` — DTOs and validation shared between auth/user services

### Authentication Flow
JWTs use **RS256** (asymmetric). The private key signs tokens in `auth_service`; the public key verifies them in any service using `shared_global::auth::user_extractors`. Axum extractors (`AuthenticatedUser`, `RequireAdmin`) pull the key from `AppState` via the `HasJwtPublicKey` trait.

### Database Migrations
Services with a DB use `sqlx::migrate!()` which runs migrations from the service's `migrations/` directory on startup.

## Environment

Secrets live in `src/backend/.env` (gitignored). Required vars for `eureka_service`:
- `JWT_PRIVATE_KEY` — base64-encoded RSA private key PEM
- `JWT_PUBLIC_KEY` — base64-encoded RSA public key PEM
- `ACCESS_TOKEN_DURATION_MINS`
- `REFRESH_TOKEN_DURATION_MINS`

Other services only need `EUREKA_URL` and (if they have a DB) `DATABASE_URL`. `SELF_URL` overrides the URL a service registers with Eureka (defaults to the Docker service hostname).

### Frontend Conventions

- All async API calls use `tryCatch(promise)` from `src/frontend/src/api/tryCatch.ts`, which returns `[data, null] | [null, errorString]` — Go-style error handling with Axios.
- Auth state lives in `AuthContext` (`context/AuthContext.tsx`). On app load, `App.tsx` attempts a silent token refresh to restore session.
- API modules in `src/frontend/src/api/`: `auth.ts`, `user.ts`, `files.ts`.
- Routes: `/` (Dashboard), `/my-files`, `/settings`, `/login`, `/register`. All except login/register are behind `ProtectedRoute`.

## Known Issues / TODOs
- `files_service` has no routes implemented yet
- `steganography_service`: only `POST /embed/video` is registered in `main.rs`; an `embed_image` handler exists in `routes/embed_image.rs` but is not wired up
- mTLS for service-to-service auth is a future TODO (currently Eureka trusts service names at face value for private key delivery)
