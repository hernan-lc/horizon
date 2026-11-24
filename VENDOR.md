# Vendored Horizon (`vendor/gaysermc`)

This branch is the panel's pinned Horizon runtime. It was branched from
`horizon-rust/horizon` `main` at `a0c8249062` (2026-11-24), the revision that
carried the following downstream fixes upstream:

- Android headless-build portability (`runtime_leases`, `engine.rs`,
  `keybinds.rs`, `entry.rs`, `window_handler.rs`, `horizon-game`);
- the generic force-stop protocol with bounded engine stdin and crash-backoff
  handling in `crates/horizon-server/src/supervisor.rs`;
- `rustix` 1.1.6 in `Cargo.lock`.

`horizon-rust/horizon` is no longer reachable on GitHub, so the panel pins this
fork instead of the original remote. Do not re-point `.gitmodules` at the old
URL: a fresh checkout would fail to initialize the submodule.

## Why this fork exists

1. **Custom auth for non-premium / test clients.** Upstream already supports an
   offline server (`online_mode = false`: no account check, offline UUIDs), and
   that is enough for test clients from the panel's Settings → Clients (Horizon)
   section. When an alternative services environment is needed, the supported
   hook is `HORIZON_API_DISCOVERY` (client and server), the counterpart of
   authlib's `-Dminecraft.api.discovery.host`. Patch points:
   - `crates/horizon-server/src/proxy/auth.rs` (`profile`, `verify_endpoint`);
   - `crates/horizon-server-files/src/profiles.rs` (`discovery_url`, `find_by_name`);
   - `crates/horizon-client-render/src/auth.rs` (client-side discovery).
2. **GayserMC → Rust mod migration.** Keep migration-specific native mods and
   any required engine changes here until they can be proposed upstream. Login
   hooks are deliberately limited: `player_login_profile` runs *after* account
   checks and before login success, so it can change profile properties (skins,
   identity) but cannot authenticate a player. See
   `docs/modding/src/how-to/entities/server-services-and-players.md` and the
   open identity/connection gaps in upstream issue #19220.

## Sync policy

- Rebase this branch onto new upstream snapshots rather than merging a moving
  target into `main` of the panel. Keep the diff small and documented.
- After changing this branch, commit the submodule gitlink in the panel repo
  and re-run `scripts/build-horizon-runtime.sh` so `REVISION` and the embedded
  `research-lock.json` match.
- Verify with `cargo check --locked -p horizon-server -p horizon-prepare --tests`
  and `cargo check --locked --profile fast -p horizon-game --no-default-features --bin horizon-client`.
