# Minna no Wadai (Everyone's Topic)

[日本語の README](README.md)

A Reddit-style thread and comment system sample implementation.
(Last updated: June 6, 2025)

## Technology Stack

### Backend

- **Language**: Rust
- **Framework**: axum
- **Database**: PostgreSQL
- **ORM**: sqlx
- **Authentication**: JWT + OAuth2 (Google, planned)
- **API Specification**: OpenAPI (utoipa)

### Frontend

- **Framework**: Next.js 15+ (App Router)
- **UI**: shadcn/ui + Tailwind CSS
- **API Client**: TanStack Query + Orval
- **Forms**: React Hook Form + Zod

## Project Structure

```
reddit-sample/
├── backend/                 # Rust (axum) backend API
│   ├── src/                 # Source code
│   │   ├── main.rs          # Entry point
│   │   ├── config.rs        # Application configuration
│   │   ├── models/          # Data models
│   │   │   ├── auth.rs
│   │   │   ├── threads.rs
│   │   │   └── users.rs
│   │   └── handlers/        # API handlers
│   │       ├── auth.rs
│   │       ├── threads.rs
│   │       └── comments.rs
│   ├── migrations/          # DB migrations
│   ├── database/            # DB configuration
│   └── Cargo.toml           # Dependencies
├── frontend/                # Next.js frontend
│   ├── src/                 # Source code
│   │   ├── app/             # App Router
│   │   │   ├── layout.tsx
│   │   │   └── page.tsx     # Homepage
│   │   ├── components/      # UI components
│   │   └── lib/             # Utilities
│   ├── generated/           # Auto-generated API code
│   └── package.json         # Dependencies
├── justfile                 # Task runner
└── README.md                # This file
```

## Features

- User authentication
- Thread posting and viewing
- Tree-structured comment system
- User profile page (displaying posted threads and comments)
- Responsive design

## Development Environment Setup

### Required Tools

- asdf
- Docker Compose
- cargo-watch
- just

### Installation Steps

#### Prerequisites

```bash
$ asdf install
$ cargo install cargo-watch
```

#### Backend

```shell
$ cd backend
$ cp .env.example .env
```

#### Frontend

```shell
$ cd frontend
$ cp .env.example .env.local
$ npm ci
```

### Starting the Development Environment

```bash
$ just dev
```

### Access Points

- **Frontend**: http://localhost:3000
- **Backend API**: http://localhost:8000
- **OpenAPI Docs**: http://localhost:8000/swagger-ui/
- **MailHog**: http://localhost:8025

## Development Workflow

1. Update API specifications in the backend
2. OpenAPI specifications are automatically updated
3. Run `npm run generate-api` in the frontend (or use `npm run generate-api:watch` for automatic generation)
4. Type-safe API clients are regenerated

## Implemented APIs

### User-Related

- User registration, login, and logout
- Google OAuth authentication
- User profile display
- Fetching threads posted by the user
- Fetching comments posted by the user
- Profile editing

### Thread-Related

- Fetching thread list
- Fetching thread details
- Creating, editing, and deleting threads

### Comment-Related

- Fetching comments for a thread
- Posting, editing, and deleting comments
- Reply comments (nested structure)

## Notes

- This project is a sample implementation for learning and demonstration purposes
- Additional security measures are required for production use
- Features are continuously being added and improved
- Please submit bug reports and feature suggestions through Issues
