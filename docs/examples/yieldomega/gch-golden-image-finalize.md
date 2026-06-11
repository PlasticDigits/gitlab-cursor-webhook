# Golden image finalize (yieldomega)

You are finishing configuration of a **GCH agent VM golden image** for the YieldOmega EVM project. The base OS packages, Docker, Rust, Foundry, Node, Playwright, glab, and Cursor CLI are already installed.

Complete the remaining setup and verify everything works. Use passwordless sudo as needed (`sudo` without a password).

## Tasks

1. **Rabby wallet extension**
   - Download the official Rabby browser extension (unpacked) into `/home/agent/.gch/extensions/rabby`
   - Use a stable release appropriate for headless Chromium/Playwright automation

2. **Anvil (local EVM)**
   - Verify Foundry (`forge`, `cast`, `anvil`) works
   - Read this repo's README and docs for Anvil setup (native `anvil` or `docker-compose.anvil.yml` if present)
   - Ensure Anvil can start and accept RPC connections for local development and Playwright dapp tests

3. **Browser profile**
   - Ensure `/home/agent/.gch/browser-profile` exists and is usable for Playwright/Chromium with the Rabby extension

4. **Verify toolchain** — run and record results:
   - `rustc --version` and `cargo --version`
   - `forge --version` and `anvil --version`
   - `docker ps` and `docker compose version`
   - `node --version` and Playwright Chromium launch smoke test
   - `agent about`
   - `glab --version`
   - Project build/test commands from this repo (e.g. `forge test`, `npm test` if documented)

5. **Write report**
   - Summarize what you installed, configured, and verified
   - List any failures or manual follow-ups for the admin
   - Save to `/home/agent/.gch/golden-image-verify.log`

## Constraints

- Do **not** run pre-snapshot cleanup (admin runs that before imaging)
- Do **not** commit or push changes unless required to verify the build; if you commit, do not add Cursor attribution trailers
- Prefer project-documented versions and paths
