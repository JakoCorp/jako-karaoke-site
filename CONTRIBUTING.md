# Contributing

## Helping

This project is open to contributions. The following guide will help set up the environment. Thank you for considering contributing.

## Setup

[`pnpm`](https://pnpm.io/) and [`Docker`](https://docs.docker.com/get-docker/) are required.

[`mkcert`](https://github.com/FiloSottile/mkcert) is required *only* for `compose.yaml`, but not `compose.dev.yaml`:

[`Rust`](https://rustup.rs/) is required only if working on the backend.

1. Fork the project

2. Clone the fork with `git`

    ```bash
    git clone https://github.com/JakoCorp/jako-karaoke-site.git
    cd jako-karaoke-site
    ```

3. Install the project from `package.json`

    ```bash
    pnpm install
    ```

    This also runs the `prepare` script, which adds [`prek`](https://prek.j178.dev/) pre-commit hooks that enforce code quality checks.

## Frontend Development

No Rust or OAuth credentials required. Docker provides the database and backend.

1. Start the database and backend

    ```bash
    docker compose -f compose.dev.yaml up --build -d
    ```

    The first run builds the backend image, which takes a bit. Subsequent runs use the cached image.

2. Seed the database (first time or after `docker compose -f compose.dev.yaml down -v` to reset)

    ```bash
    docker compose -f compose.dev.yaml exec -T db mariadb -u root -ppassword jako < dev/seed.sql
    ```

3. Start the frontend dev server

    ```bash
    pnpm dev
    ```

4. Open the site and click **Dev Login** to authenticate as the seeded admin user.

## Backend Development

See the [backend](./backend).

## Checks and Linting

While `prek` should automatically run checks and linting, here are the CLI commands to run them manually:

Frontend (from either root or in `frontend` dir):

```bash
pnpm run lint
```

Backend:

```bash
cd backend
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
```
