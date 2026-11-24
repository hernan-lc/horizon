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

## GayserMC auth plugin (`mods/gaysermc-auth`)

A native mod implementing `player_login` (name allowlist from
`GAYSERMC_ALLOWED_NAMES`) and `player_login_profile` (a deterministic skin for
offline clients). It builds against `crates/horizon-mod-api` and its two unit
tests pass with the mod API's dispatch harness.

**Verified limitation:** both login hooks are dispatched only from the LAN /
integrated-host path:
- `crates/horizon-client/src/game/network/lan/mod.rs:2233` calls `native_player_login`;
- `crates/horizon-client/src/lan/conn.rs:1345` calls `native_player_login_profile`.

The dedicated server's login gate (`game/interface/dedicated/mod.rs`, the
`LoginGate` closure) only checks whitelist/bans/player count and never calls
either hook. So **the mod does not run on a panel-hosted `horizon-server`**.
Making custom auth work there requires wiring a hook into that gate in this
fork; it cannot be done from a mod alone.

## Sync policy

- Rebase this branch onto new upstream snapshots rather than merging a moving
  target into `main` of the panel. Keep the diff small and documented.
- After changing this branch, commit the submodule gitlink in the panel repo
  and re-run `scripts/build-horizon-runtime.sh` so `REVISION` and the embedded
  `research-lock.json` match.
- Verify with `cargo check --locked -p horizon-server -p horizon-prepare --tests`
  and `cargo check --locked --profile fast -p horizon-game --no-default-features --bin horizon-client`.
