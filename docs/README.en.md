<p align="center">
  <a href="https://minwada.com">
    <img src="./images/logo.png" alt="みんなの話題" width="300" height="300">
  </a>
</p>

# 🗣️ [Minna no Wadai (Everyone's Topic)](https://minwada.com)

[![Rust](https://img.shields.io/badge/rust-1.87.0-orange.svg?logo=rust)](https://www.rust-lang.org)
[![Next.js](https://img.shields.io/badge/next.js-15+-black.svg?logo=next.js)](https://nextjs.org/)

[日本語の README](/README.md)

> A Reddit-style thread and comment system sample implementation. Post topics freely and enjoy communication through comments.

> [!WARNING]
> Please note that most of the code in this project, including this README, was generated by AI and may not always be correct.

## ✨ Features

- 👤 User authentication (JWT)
- 📝 Thread posting and viewing
- 💬 Tree-structured comment system
- 👨‍💻 User profile page (displaying posted threads and comments)
- 📱 Responsive design

## 🛠️ Technology Stack

### Backend

- **Language**: [Rust](https://www.rust-lang.org/) 🦀
- **Framework**: [axum](https://github.com/tokio-rs/axum) ⚡
- **Database**: [PostgreSQL](https://www.postgresql.org/) 🐘
- **ORM**: [sqlx](https://github.com/launchbadge/sqlx) 📊
- **Authentication**: JWT + OAuth2 (Google, planned) 🔐
- **API Specification**: [OpenAPI](https://www.openapis.org/) ([utoipa](https://github.com/juhaku/utoipa)) 📚

### Frontend

- **Framework**: [Next.js](https://nextjs.org/) 15+ (App Router) ⚛️
- **UI**: [shadcn/ui](https://ui.shadcn.com/) + [Tailwind CSS](https://tailwindcss.com/) 🎨
- **API Client**: [TanStack Query](https://tanstack.com/query) + [Orval](https://orval.dev/) 🔄
- **Forms**: [React Hook Form](https://react-hook-form.com/) + [Zod](https://zod.dev/) 📋

## 📂 Project Structure

```
minwada/
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

## 🚀 Development Environment Setup

### Required Tools

- [asdf](https://asdf-vm.com/) - Version management
- [Docker Compose](https://docs.docker.com/compose/) - Container management
- [cargo-watch](https://github.com/watchexec/cargo-watch) - Rust hot reloading
- [just](https://github.com/casey/just) - Task runner

### Installation Steps

#### 1️⃣ Prerequisites

```bash
$ asdf install
$ cargo install cargo-watch
```

#### 2️⃣ Backend

```shell
$ cd backend
$ cp .env.example .env  # Set environment variables
```

#### 3️⃣ Frontend

```shell
$ cd frontend
$ cp .env.example .env.local  # Set environment variables
$ npm ci  # Install dependencies
```

### Starting the Development Environment

```bash
$ just dev  # Starts both backend and frontend
```

### Access Points

- 🌐 **Frontend**: http://localhost:3000
- 🔌 **Backend API**: http://localhost:8000
- 📘 **OpenAPI Docs**: http://localhost:8000/swagger-ui/
- 📧 **MailHog**: http://localhost:8025

## 💻 Development Workflow

1. Update API specifications in the backend
2. OpenAPI specifications are automatically updated
3. Run `npm run generate-api` in the frontend (or use `npm run generate-api:watch` for automatic generation)
4. Type-safe API clients are regenerated

## 🔍 Implemented APIs

<details>
<summary>👤 User-Related</summary>

- User registration, login, and logout
- Google OAuth authentication
- User profile display
- Fetching threads posted by the user
- Fetching comments posted by the user
- Profile editing
</details>

<details>
<summary>📋 Thread-Related</summary>

- Fetching thread list
- Fetching thread details
- Creating, editing, and deleting threads
</details>

<details>
<summary>💬 Comment-Related</summary>

- Fetching comments for a thread
- Posting, editing, and deleting comments
- Reply comments (nested structure)
</details>

## ⚠️ Notes

- This project is a sample implementation for learning and demonstration purposes
- Additional security measures are required for production use
- Features are continuously being added and improved
- Please submit bug reports and feature suggestions through [Issues](https://github.com/y-temp4/minwada/issues)
