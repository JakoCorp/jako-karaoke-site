# Contributing

## Helping

This project is open to contributions. The following guide will help set up the environment. Thank you for considering contributing.

## Setup

[`pnpm`](https://pnpm.io/) is required.

[`Rust`](https://rustup.rs/) is required if working on the backend.

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

4. OPTIONAL: If also working on the backend

    ```bash
    cd backend
    cargo install
    ```

Step 3 will run the `prepare` project script. This adds [`prek`](https://prek.j178.dev/) pre-commit hooks that enforce code quality checks on the project.

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
