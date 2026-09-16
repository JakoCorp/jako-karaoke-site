# Jako Karaoke Site Backend

Backend API server for the karaoke site. Run with `cargo run`.

## Config

Configuration is read from environment variables at startup. For local development, place them in `backend/.cargo/config.toml` under `[env]`.

| Variable | Description |
| --- | --- |
| `PORT` | Port to listen on. Defaults to `3000`. |
| `DATABASE_URL` | MariaDB connection string (e.g. `mysql://user:pass@host/dbname`) |
| `STORAGE_PATH` | Absolute path where uploaded media files are stored on disk |
| `STORAGE_BASE_URL` | Public base HTTP/HTTPS URL that maps to `STORAGE_PATH` |
| `BASE_URL` | Public base HTTP/HTTPS URL of this backend server. Used to build OAuth callback URLs. |
| `FRONTEND_URL` | Public base HTTP/HTTPS URL of frontend app. Used for CORS and redirects. |
| `TWITCH_CLIENT_ID` | Twitch OAuth application client ID |
| `TWITCH_CLIENT_SECRET` | Twitch OAuth application client secret |
| `DISCORD_CLIENT_ID` | Discord OAuth application client ID |
| `DISCORD_CLIENT_SECRET` | Discord OAuth application client secret |
| `RUST_LOG` | Log filter (e.g. `server=debug,tower_http=debug`) |
| `DEV_AUTH` | Set to `true` or `1` to enable `GET /auth/dev-login`. Must never be set in production. |

## Database

MariaDB is used for this backend. Migration files live in `crates/db/migrations/` and are applied automatically on startup.

## OAuth

### Twitch

Create a Twitch app at `https://dev.twitch.tv/console` under `Applications`.

- For `OAuth Redirect URLs` use `{BASE_URL}/auth/twitch/callback`
- `Cateogry` is `Website Integration`
- Copy the `Client ID` and generate + copy a `Client Secret`

For further information or if it has changed, [see their documentation](https://dev.twitch.tv/docs/authentication/).

### Discord

Create a Discord OAuth2 app at `https://discord.com/developers/applications` (configure under `OAuth2` tab)

- Under `General Information` tab, fill in `Name`, `Terms of Service URL`, `Privacy Policy URL`
- Under `OAuth2` tab, for `Redirects` use `{BASE_URL}/auth/discord/callback`
- Under `OAuth2` tab, copy the `Client ID` and generate + copy a `Client Secret`

For further information or if it has changed, [see their documentation](https://docs.discord.com/developers/topics/oauth2).

## Crates

| Crate | Description |
| --- | --- |
| `server` | Axum HTTP server, route handlers, auth, and middleware |
| `db` | Database models, queries, and migrations |
| `api-types` | Shared request and response types |

The API documentation is at `{BASE_URL}/docs`.

## Misc

- This backend assumes that `FRONTEND_URL` is SPA (Single Page App) since it uses that for the CORS assignment (with `credentials: 'include'`)
