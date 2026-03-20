# 🌍 Cloudflare DDNS

Access your home network remotely via a custom domain name without a static IP!

A feature-complete dynamic DNS client written in Rust with support for **multiple DNS backends** — Cloudflare and Technitium DNS Server. The **smallest and most memory-efficient** open-source DDNS Docker image available — **~1.9 MB image size** and **~3.5 MB RAM** at runtime, smaller and leaner than Go-based alternatives. Built as a fully static binary from scratch with zero runtime dependencies.

Configure everything with environment variables. Supports notifications, heartbeat monitoring, WAF list management (Cloudflare), flexible scheduling, and more.

[![Docker Pulls](https://img.shields.io/docker/pulls/timothyjmiller/cloudflare-ddns?style=flat&logo=docker&label=pulls)](https://hub.docker.com/r/timothyjmiller/cloudflare-ddns) [![Docker Image Size](https://img.shields.io/docker/image-size/timothyjmiller/cloudflare-ddns/latest?style=flat&logo=docker&label=image%20size)](https://hub.docker.com/r/timothyjmiller/cloudflare-ddns)

## ✨ Features

- 🔌 **Multiple DNS backends** — Cloudflare and Technitium DNS Server
- 🔍 **Multiple IP detection providers** — Cloudflare Trace, Cloudflare DNS-over-HTTPS, ipify, local interface, custom URL, or static IPs
- 📡 **IPv4 and IPv6** — Full dual-stack support with independent provider configuration
- 🌐 **Multiple domains and zones** — Update any number of domains across multiple zones
- 🃏 **Wildcard domains** — Support for `*.example.com` records
- 🌍 **Internationalized domain names** — Full IDN/punycode support (e.g. `münchen.de`)
- 🛡️ **WAF list management** — Automatically update Cloudflare WAF IP lists
- 🔔 **Notifications** — Shoutrrr-compatible notifications (Discord, Slack, Telegram, Gotify, Pushover, generic webhooks)
- 💓 **Heartbeat monitoring** — Healthchecks.io and Uptime Kuma integration
- ⏱️ **Cron scheduling** — Flexible update intervals via cron expressions
- 🧪 **Dry-run mode** — Preview changes without modifying DNS records
- 🧹 **Graceful shutdown** — Signal handling (SIGINT/SIGTERM) with optional DNS record cleanup
- 💬 **Record comments** — Tag managed Cloudflare records with comments for identification
- 🎯 **Managed record regex** — Control which Cloudflare records the tool manages via regex matching
- 🎨 **Pretty output with emoji** — Configurable emoji and verbosity levels
- 🔒 **Zero-log IP detection** — Uses Cloudflare's [cdn-cgi/trace](https://www.cloudflare.com/cdn-cgi/trace) by default
- 🏠 **CGNAT-aware local detection** — Filters out shared address space (100.64.0.0/10) and private ranges
- 🚫 **Cloudflare IP rejection** — Automatically rejects Cloudflare anycast IPs to prevent incorrect DNS updates
- 🤏 **Tiny static binary** — ~1.9 MB Docker image built from scratch, zero runtime dependencies

## 🚀 Quick Start

### Cloudflare

```bash
docker run -d \
  --name cloudflare-ddns \
  --restart unless-stopped \
  --network host \
  -e CLOUDFLARE_API_TOKEN=your-api-token \
  -e DOMAINS=example.com,www.example.com \
  ghcr.io/joenorth/cloudflare-ddns:latest
```

### Technitium

```bash
docker run -d \
  --name cloudflare-ddns \
  --restart unless-stopped \
  --network host \
  -e BACKEND=technitium \
  -e TECHNITIUM_URL=http://technitium.local:5380 \
  -e TECHNITIUM_TOKEN=your-api-token \
  -e DOMAINS=example.com,www.example.com \
  ghcr.io/joenorth/cloudflare-ddns:latest
```

The container detects your public IP and updates DNS records for your domains every 5 minutes.

> ⚠️ `--network host` is required to detect IPv6 addresses. If you only need IPv4, you can omit it and set `IP6_PROVIDER=none`.

---

## 🔌 Backends

| Variable | Default | Description |
|----------|---------|-------------|
| `BACKEND` | `cloudflare` | DNS backend to use: `cloudflare` or `technitium` |

The `BACKEND` variable selects which DNS provider is used. When omitted, Cloudflare is used. Each backend has its own authentication requirements and feature set — see below.

Setting `BACKEND=technitium` (or setting `TECHNITIUM_URL`/`TECHNITIUM_TOKEN`) activates env-var configuration mode without requiring a `CLOUDFLARE_API_TOKEN`. You do **not** need a Cloudflare account to use the Technitium backend.

### ☁️ Cloudflare

The default backend. Manages DNS records via the Cloudflare API with support for proxied records, record comments, and WAF list management.

#### 🔑 Authentication

| Variable | Description |
|----------|-------------|
| `CLOUDFLARE_API_TOKEN` | API token with "Edit DNS" capability |
| `CLOUDFLARE_API_TOKEN_FILE` | Path to a file containing the API token (Docker secrets compatible) |

To generate an API token, go to your [Cloudflare Profile](https://dash.cloudflare.com/profile/api-tokens) and create a token capable of **Edit DNS**.

#### ☁️ Proxied Records

| Variable | Default | Description |
|----------|---------|-------------|
| `PROXIED` | `false` | Expression controlling which domains are proxied through Cloudflare |

The `PROXIED` variable supports boolean expressions:

| Expression | Meaning |
|------------|---------|
| `true` | ☁️ Proxy all domains |
| `false` | 🔓 Don't proxy any domains |
| `is(example.com)` | 🎯 Only proxy `example.com` |
| `sub(cdn.example.com)` | 🌳 Proxy `cdn.example.com` and its subdomains |
| `is(a.com) \|\| is(b.com)` | 🔀 Proxy `a.com` or `b.com` |
| `!is(vpn.example.com)` | 🚫 Proxy everything except `vpn.example.com` |

Operators: `is()`, `sub()`, `!`, `&&`, `||`, `()`

#### 💬 Record Comments

| Variable | Default | Description |
|----------|---------|-------------|
| `RECORD_COMMENT` | (empty) | Comment attached to managed DNS records |
| `MANAGED_RECORDS_COMMENT_REGEX` | (empty) | Regex to identify which records are managed (empty = all) |

#### 🛡️ WAF Lists

| Variable | Default | Description |
|----------|---------|-------------|
| `WAF_LISTS` | (empty) | Comma-separated WAF lists in `account-id/list-name` format |
| `WAF_LIST_DESCRIPTION` | (empty) | Description for managed WAF lists |
| `WAF_LIST_ITEM_COMMENT` | (empty) | Comment for WAF list items |
| `MANAGED_WAF_LIST_ITEMS_COMMENT_REGEX` | (empty) | Regex to identify managed WAF list items |

WAF list names must match the pattern `[a-z0-9_]+`.

#### 🚫 Cloudflare IP Rejection

| Variable | Default | Description |
|----------|---------|-------------|
| `REJECT_CLOUDFLARE_IPS` | `true` | Reject detected IPs that fall within Cloudflare's IP ranges |

Some IP detection providers occasionally return a Cloudflare anycast IP instead of your real public IP. When this happens, your DNS record gets updated to point at Cloudflare infrastructure rather than your actual address.

By default, each update cycle fetches [Cloudflare's published IP ranges](https://www.cloudflare.com/ips/) and skips any detected IP that falls within them. A warning is logged for every rejected IP. If the ranges cannot be fetched, the update is skipped entirely to prevent writing a Cloudflare IP.

To disable this protection, set `REJECT_CLOUDFLARE_IPS=false`.

### 🖥️ Technitium

[Technitium DNS Server](https://technitium.com/dns/) is a self-hosted authoritative and recursive DNS server. This backend manages A and AAAA records via Technitium's HTTP API.

#### 🔑 Authentication

| Variable | Required | Description |
|----------|----------|-------------|
| `TECHNITIUM_URL` | **yes** | Base URL of the Technitium DNS Server (e.g. `http://technitium.local:5380`) |
| `TECHNITIUM_TOKEN` | **yes** (or `_FILE`) | API token for Technitium DNS Server |
| `TECHNITIUM_TOKEN_FILE` | **yes** (or `TOKEN`) | Path to a file containing the API token (Docker secrets compatible) |

All three variables (`BACKEND`, `TECHNITIUM_URL`, and one of `TECHNITIUM_TOKEN`/`TECHNITIUM_TOKEN_FILE`) must be set. If any required variable is missing, the program exits with an error.

To generate an API token, open your Technitium DNS Server web console, go to **Administration** > **Sessions**, and create an API token.

#### 📋 Zone Auto-Discovery

Zones are resolved automatically. When updating `sub.example.com`, the client walks parent domains to find a matching zone configured in your Technitium server (e.g., `example.com`). No zone IDs are needed.

#### ⚠️ Behavior Details

- **TTL**: Technitium does not have a Cloudflare-style "auto" TTL. If `TTL` is set to `1` (auto), it is explicitly mapped to **300 seconds**. Set `TTL` to an explicit value (e.g. `60`, `300`, `3600`) to control the TTL directly.
- **Ignored variables**: The following environment variables are silently ignored (not an error) when `BACKEND=technitium`: `PROXIED`, `RECORD_COMMENT`, `MANAGED_RECORDS_COMMENT_REGEX`, `WAF_LISTS`, `WAF_LIST_DESCRIPTION`, `WAF_LIST_ITEM_COMMENT`, `MANAGED_WAF_LIST_ITEMS_COMMENT_REGEX`, `REJECT_CLOUDFLARE_IPS`. Setting them will not produce warnings or errors — they simply have no effect.
- **WAF lists**: WAF list management is not available with the Technitium backend. The `WAF_LISTS` variable is ignored; no WAF-related API calls are made.
- **Record overwrite behavior**: When updating a domain, the first IP address **overwrites** all existing records of that type (A or AAAA). Additional IPs are **added** alongside the first. When the detected IP list is empty, **all** records of that type for the domain are deleted.
- **Zone resolution**: Zones are resolved by listing all zones from the Technitium server and matching the domain's parent. For example, updating `sub.example.com` will match a zone named `example.com`. If no matching zone is found, the update fails with an error.

#### 🐳 Docker Compose Example

```yml
version: '3.9'
services:
  cloudflare-ddns:
    image: ghcr.io/joenorth/cloudflare-ddns:latest
    container_name: cloudflare-ddns
    security_opt:
      - no-new-privileges:true
    network_mode: 'host'
    environment:
      - BACKEND=technitium
      - TECHNITIUM_URL=http://technitium.local:5380
      - TECHNITIUM_TOKEN=your-api-token
      - DOMAINS=example.com,www.example.com
      - IP6_PROVIDER=none
    restart: unless-stopped
```

---

## 🌐 Shared Configuration

The following settings apply to all backends.

### 🌐 Domains

| Variable | Description |
|----------|-------------|
| `DOMAINS` | Comma-separated list of domains to update for both IPv4 and IPv6 |
| `IP4_DOMAINS` | Comma-separated list of IPv4-only domains |
| `IP6_DOMAINS` | Comma-separated list of IPv6-only domains |

Wildcard domains are supported: `*.example.com`

At least one of `DOMAINS`, `IP4_DOMAINS`, `IP6_DOMAINS`, or `WAF_LISTS` (Cloudflare only) must be set.

### 🔍 IP Detection Providers

| Variable | Default | Description |
|----------|---------|-------------|
| `IP4_PROVIDER` | `ipify` | IPv4 detection method |
| `IP6_PROVIDER` | `cloudflare.trace` | IPv6 detection method |

Available providers:

| Provider | Description |
|----------|-------------|
| `cloudflare.trace` | 🔒 Cloudflare's `/cdn-cgi/trace` endpoint (default, zero-log) |
| `cloudflare.doh` | 🌐 Cloudflare DNS-over-HTTPS (`whoami.cloudflare` TXT query) |
| `ipify` | 🌎 ipify.org API |
| `local` | 🏠 Local IP via system routing table (no network traffic, CGNAT-aware) |
| `local.iface:<name>` | 🔌 IP from a specific network interface (e.g., `local.iface:eth0`) |
| `url:<url>` | 🔗 Custom HTTP(S) endpoint that returns an IP address |
| `literal:<ips>` | 📌 Static IP addresses (comma-separated) |
| `none` | 🚫 Disable this IP type |

### ⏱️ Scheduling

| Variable | Default | Description |
|----------|---------|-------------|
| `UPDATE_CRON` | `@every 5m` | Update schedule |
| `UPDATE_ON_START` | `true` | Run an update immediately on startup |
| `DELETE_ON_STOP` | `false` | Delete managed DNS records on shutdown |

Schedule formats:

- `@every 5m` — Every 5 minutes
- `@every 1h` — Every hour
- `@every 30s` — Every 30 seconds
- `@once` — Run once and exit

When `UPDATE_CRON=@once`, `UPDATE_ON_START` must be `true` and `DELETE_ON_STOP` must be `false`.

### 📝 TTL

| Variable | Default | Description |
|----------|---------|-------------|
| `TTL` | `1` (auto) | DNS record TTL in seconds (1=auto, or 30-86400) |

TTL behavior differs by backend:

| Backend | `TTL=1` behavior | Explicit TTL (e.g. `300`) |
|---------|-------------------|---------------------------|
| Cloudflare | Cloudflare "automatic" TTL | Used as-is |
| Technitium | Mapped to **300 seconds** | Used as-is |

### ⏳ Timeouts

| Variable | Default | Description |
|----------|---------|-------------|
| `DETECTION_TIMEOUT` | `5s` | Timeout for IP detection requests |
| `UPDATE_TIMEOUT` | `30s` | Timeout for DNS backend API requests |

### 🖥️ Output

| Variable | Default | Description |
|----------|---------|-------------|
| `EMOJI` | `true` | Use emoji in output messages |
| `QUIET` | `false` | Suppress informational output |

### 🏁 CLI Flags

| Flag | Description |
|------|-------------|
| `--dry-run` | 🧪 Preview changes without modifying DNS records |
| `--repeat` | 🔁 Run continuously (legacy config mode only; env var mode uses `UPDATE_CRON`) |

---

## 🔗 Integrations

### 🔔 Notifications (Shoutrrr)

| Variable | Description |
|----------|-------------|
| `SHOUTRRR` | Newline-separated list of notification service URLs |

Supported services:

| Service | URL format |
|---------|------------|
| 💬 Discord | `discord://token@webhook-id` |
| 📨 Slack | `slack://token-a/token-b/token-c` |
| ✈️ Telegram | `telegram://bot-token@telegram?chats=chat-id` |
| 📡 Gotify | `gotify://host/path?token=app-token` |
| 📲 Pushover | `pushover://user-key@api-token` |
| 🌐 Generic webhook | `generic://host/path` or `generic+https://host/path` |

Notifications are sent when DNS records are updated, created, deleted, or when errors occur.

### 💓 Heartbeat Monitoring

| Variable | Description |
|----------|-------------|
| `HEALTHCHECKS` | Healthchecks.io ping URL |
| `UPTIMEKUMA` | Uptime Kuma push URL |

Heartbeats are sent after each update cycle. On failure, a fail signal is sent. On shutdown, an exit signal is sent.

### 🐳 Docker Label Discovery

Automatically discover domains from running Docker containers. Containers are identified by the `cloudflare-ddns.domain` label, which can contain a comma-separated list of domains.

| Variable | Default | Description |
|----------|---------|-------------|
| `DOCKER_LABEL_ENABLED` | `false` | Enable Docker label discovery |
| `DOCKER_SOCKET` | (auto) | Custom Docker socket path (e.g. `/var/run/docker.sock`) |

Add the label to your containers:

```yml
services:
  my-app:
    image: my-app:latest
    labels:
      cloudflare-ddns.domain: "app.example.com,api.example.com"
```

Discovered domains are merged with any statically configured `DOMAINS` each update cycle. The Docker socket must be accessible to the cloudflare-ddns container:

```yml
services:
  cloudflare-ddns:
    image: ghcr.io/joenorth/cloudflare-ddns:latest
    environment:
      - CLOUDFLARE_API_TOKEN=your-api-token
      - DOCKER_LABEL_ENABLED=true
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

### 🖥️ Proxmox VM Discovery

Automatically discover domains from Proxmox VE virtual machines. VMs are identified by a configurable tag, and the VM name is used as the domain name. IP addresses are read from the QEMU guest agent via the NIC attached to `vmbr0`.

| Variable | Default | Description |
|----------|---------|-------------|
| `PROXMOX_ENABLED` | `false` | Enable Proxmox VM discovery |
| `PROXMOX_API_URL` | — | Proxmox API URL (e.g. `https://pve.example.com:8006`) |
| `PROXMOX_API_TOKEN` | — | API token (e.g. `user@pam!token=secret-uuid`) |
| `PROXMOX_TAG` | `dns` | Only discover VMs with this tag |

#### API Token Permissions

Create a dedicated role with the minimum read-only privileges:

**Proxmox VE 8.x:**

| Path | Privilege |
|------|-----------|
| `/` | `Sys.Audit` |
| `/vms` | `VM.Audit`, `VM.Monitor` |

**Proxmox VE 9.x+:**

| Path | Privilege |
|------|-----------|
| `/` | `Sys.Audit` |
| `/vms` | `VM.Audit`, `VM.GuestAgent` |

> On PVE 9.x+, prefer `VM.GuestAgent` over `VM.Monitor` — it is more narrowly scoped and does not grant full QEMU monitor access.

Example setup:

```bash
# Create a read-only role (PVE 8.x)
pveum role add CloudflareDDNS -privs "Sys.Audit,VM.Audit,VM.Monitor"

# Create a user and API token
pveum user add cloudflare-ddns@pve
pveum aclmod / -user cloudflare-ddns@pve -role CloudflareDDNS
pveum user token add cloudflare-ddns@pve ddns -privsep 0
```

#### Requirements

- The **QEMU guest agent** must be installed and running inside each VM
- VMs must have a NIC attached to **vmbr0** (the bridge used to find the IP)
- The VM **name** is used as the domain name (e.g., name a VM `web.example.com`)
- VMs must be **tagged** with the configured tag (default: `dns`)

---

## 📋 All Environment Variables

### General

| Variable | Default | Description |
|----------|---------|-------------|
| `BACKEND` | `cloudflare` | 🔌 DNS backend (`cloudflare` or `technitium`) |
| `DOMAINS` | — | 🌐 Domains for both IPv4 and IPv6 |
| `IP4_DOMAINS` | — | 4️⃣ IPv4-only domains |
| `IP6_DOMAINS` | — | 6️⃣ IPv6-only domains |
| `IP4_PROVIDER` | `ipify` | 🔍 IPv4 detection provider |
| `IP6_PROVIDER` | `cloudflare.trace` | 🔍 IPv6 detection provider |
| `UPDATE_CRON` | `@every 5m` | ⏱️ Update schedule |
| `UPDATE_ON_START` | `true` | 🚀 Update on startup |
| `DELETE_ON_STOP` | `false` | 🧹 Delete records on shutdown |
| `TTL` | `1` | ⏳ DNS record TTL |
| `DETECTION_TIMEOUT` | `5s` | ⏳ IP detection timeout |
| `UPDATE_TIMEOUT` | `30s` | ⏳ API request timeout |
| `EMOJI` | `true` | 🎨 Enable emoji output |
| `QUIET` | `false` | 🤫 Suppress info output |

### Cloudflare Backend

| Variable | Default | Description |
|----------|---------|-------------|
| `CLOUDFLARE_API_TOKEN` | — | 🔑 API token |
| `CLOUDFLARE_API_TOKEN_FILE` | — | 📄 Path to API token file |
| `PROXIED` | `false` | ☁️ Proxied expression |
| `RECORD_COMMENT` | — | 💬 DNS record comment |
| `MANAGED_RECORDS_COMMENT_REGEX` | — | 🎯 Managed records regex |
| `WAF_LISTS` | — | 🛡️ WAF lists to manage |
| `WAF_LIST_DESCRIPTION` | — | 📝 WAF list description |
| `WAF_LIST_ITEM_COMMENT` | — | 💬 WAF list item comment |
| `MANAGED_WAF_LIST_ITEMS_COMMENT_REGEX` | — | 🎯 Managed WAF items regex |
| `REJECT_CLOUDFLARE_IPS` | `true` | 🚫 Reject Cloudflare anycast IPs |

### Technitium Backend

| Variable | Default | Description |
|----------|---------|-------------|
| `TECHNITIUM_URL` | — | 🖥️ Technitium DNS Server URL |
| `TECHNITIUM_TOKEN` | — | 🔑 API token |
| `TECHNITIUM_TOKEN_FILE` | — | 📄 Path to API token file |

### Integrations

| Variable | Default | Description |
|----------|---------|-------------|
| `HEALTHCHECKS` | — | 💓 Healthchecks.io URL |
| `UPTIMEKUMA` | — | 💓 Uptime Kuma URL |
| `SHOUTRRR` | — | 🔔 Notification URLs (newline-separated) |
| `DOCKER_LABEL_ENABLED` | `false` | 🐳 Enable Docker label discovery |
| `DOCKER_SOCKET` | (auto) | 🐳 Custom Docker socket path |
| `PROXMOX_ENABLED` | `false` | 🖥️ Enable Proxmox VM discovery |
| `PROXMOX_API_URL` | — | 🖥️ Proxmox API URL |
| `PROXMOX_API_TOKEN` | — | 🖥️ Proxmox API token |
| `PROXMOX_TAG` | `dns` | 🖥️ VM tag to filter by |

---

## 🚢 Deployment

### 🐳 Docker Compose (Cloudflare)

```yml
version: '3.9'
services:
  cloudflare-ddns:
    image: ghcr.io/joenorth/cloudflare-ddns:latest
    container_name: cloudflare-ddns
    security_opt:
      - no-new-privileges:true
    network_mode: 'host'
    environment:
      - CLOUDFLARE_API_TOKEN=your-api-token
      - DOMAINS=example.com,www.example.com
      - PROXIED=true
      - IP6_PROVIDER=none
      - HEALTHCHECKS=https://hc-ping.com/your-uuid
    restart: unless-stopped
```

### 🐳 Docker Compose (Technitium)

```yml
version: '3.9'
services:
  cloudflare-ddns:
    image: ghcr.io/joenorth/cloudflare-ddns:latest
    container_name: cloudflare-ddns
    security_opt:
      - no-new-privileges:true
    network_mode: 'host'
    environment:
      - BACKEND=technitium
      - TECHNITIUM_URL=http://technitium.local:5380
      - TECHNITIUM_TOKEN=your-api-token
      - DOMAINS=example.com,www.example.com
      - IP6_PROVIDER=none
    restart: unless-stopped
```

> ⚠️ Docker requires `network_mode: host` to access the IPv6 public address.

### ☸️ Kubernetes

The included manifest uses the legacy JSON config mode. Create a secret containing your `config.json` and apply:

```bash
kubectl create secret generic config-cloudflare-ddns --from-file=config.json -n ddns
kubectl apply -f k8s/cloudflare-ddns.yml
```

### 🐧 Linux + Systemd

1. Build and install:

```bash
cargo build --release
sudo cp target/release/cloudflare-ddns /usr/local/bin/
```

2. Copy the systemd units from the `systemd/` directory:

```bash
sudo cp systemd/cloudflare-ddns.service /etc/systemd/system/
sudo cp systemd/cloudflare-ddns.timer /etc/systemd/system/
```

3. Place a `config.json` at `/etc/cloudflare-ddns/config.json` (the systemd service uses legacy config mode).

4. Enable the timer:

```bash
sudo systemctl enable --now cloudflare-ddns.timer
```

The timer runs the service every 15 minutes (configurable in `cloudflare-ddns.timer`).

## 🔨 Building from Source

```bash
cargo build --release
```

The binary is at `target/release/cloudflare-ddns`.

### 🐳 Docker builds

```bash
# Single architecture (linux/amd64)
./scripts/docker-build.sh

# Multi-architecture (linux/amd64, linux/arm64, linux/ppc64le)
./scripts/docker-build-all.sh
```

## 💻 Supported Platforms

- 🐳 [Docker](https://docs.docker.com/get-docker/) (amd64, arm64, ppc64le)
- 🐙 [Docker Compose](https://docs.docker.com/compose/install/)
- ☸️ [Kubernetes](https://kubernetes.io/docs/tasks/tools/)
- 🐧 [Systemd](https://www.freedesktop.org/wiki/Software/systemd/)
- 🍎 macOS, 🪟 Windows, 🐧 Linux — anywhere Rust compiles

---

## 📁 Legacy JSON Config File

For backwards compatibility, cloudflare-ddns still supports configuration via a `config.json` file. This mode activates automatically when **none** of the following environment variables are set: `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_API_TOKEN_FILE`, `DOMAINS`, `IP4_DOMAINS`, `IP6_DOMAINS`, `TECHNITIUM_URL`, `TECHNITIUM_TOKEN`, `BACKEND`. Legacy mode always uses the **Cloudflare backend** — the Technitium backend is only available in env-var mode.

### 🚀 Quick Start

```bash
cp config-example.json config.json
# Edit config.json with your values
cloudflare-ddns
```

### 🔑 Authentication

Use either an API token (recommended) or a legacy API key:

```json
"authentication": {
  "api_token": "Your cloudflare API token with Edit DNS capability"
}
```

Or with a legacy API key:

```json
"authentication": {
  "api_key": {
    "api_key": "Your cloudflare API Key",
    "account_email": "The email address you use to sign in to cloudflare"
  }
}
```

### 📡 IPv4 and IPv6

Some ISP provided modems only allow port forwarding over IPv4 or IPv6. Disable the interface that is not accessible:

```json
"a": true,
"aaaa": true
```

### ⚙️ Config Options

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cloudflare` | array | required | List of zone configurations |
| `a` | bool | `true` | Enable IPv4 (A record) updates |
| `aaaa` | bool | `true` | Enable IPv6 (AAAA record) updates |
| `purgeUnknownRecords` | bool | `false` | Delete stale/duplicate DNS records |
| `ttl` | int | `300` | DNS record TTL in seconds (30-86400, values < 30 become auto) |
| `ip4_provider` | string | `"cloudflare.trace"` | IPv4 detection provider (same values as `IP4_PROVIDER` env var) |
| `ip6_provider` | string | `"cloudflare.trace"` | IPv6 detection provider (same values as `IP6_PROVIDER` env var) |

### 🚫 Cloudflare IP Rejection (Legacy Mode)

Cloudflare IP rejection is enabled by default in legacy mode too. To disable it, set `REJECT_CLOUDFLARE_IPS=false` alongside your `config.json`:

```bash
REJECT_CLOUDFLARE_IPS=false cloudflare-ddns
```

Or in Docker Compose:

```yml
environment:
  - REJECT_CLOUDFLARE_IPS=false
volumes:
  - ./config.json:/config.json
```

### 🔍 IP Detection (Legacy Mode)

Legacy mode now uses the same shared provider abstraction as environment variable mode. By default it uses the `cloudflare.trace` provider, which builds an IP-family-bound HTTP client (`0.0.0.0` for IPv4, `[::]` for IPv6) to guarantee the correct address family on dual-stack hosts.

You can override the detection method per address family with `ip4_provider` and `ip6_provider` in your `config.json`. Supported values are the same as the `IP4_PROVIDER` / `IP6_PROVIDER` environment variables: `cloudflare.trace`, `cloudflare.doh`, `ipify`, `local`, `local.iface:<name>`, `url:<https://...>`, `none`.

Set a provider to `"none"` to disable detection for that address family (overrides `a`/`aaaa`):

```json
{
  "a": true,
  "aaaa": true,
  "ip4_provider": "cloudflare.trace",
  "ip6_provider": "none"
}
```

Each zone entry contains:

| Key | Type | Description |
|-----|------|-------------|
| `authentication` | object | API token or API key credentials |
| `zone_id` | string | Cloudflare zone ID (found in zone dashboard) |
| `subdomains` | array | Subdomain entries to update |
| `proxied` | bool | Default proxied status for subdomains in this zone |

Subdomain entries can be a simple string or a detailed object:

```json
"subdomains": [
  "",
  "@",
  "www",
  { "name": "vpn", "proxied": true }
]
```

Use `""` or `"@"` for the root domain. Do not include the base domain name.

### 🔄 Environment Variable Substitution

In the legacy config file, values can reference environment variables with the `CF_DDNS_` prefix:

```json
{
  "cloudflare": [{
    "authentication": {
      "api_token": "${CF_DDNS_API_TOKEN}"
    },
    ...
  }]
}
```

### 📠 Example: Multiple Subdomains

```json
{
  "cloudflare": [
    {
      "authentication": {
        "api_token": "your-api-token"
      },
      "zone_id": "your_zone_id",
      "subdomains": [
        { "name": "", "proxied": true },
        { "name": "www", "proxied": true },
        { "name": "vpn", "proxied": false }
      ]
    }
  ],
  "a": true,
  "aaaa": true,
  "purgeUnknownRecords": false,
  "ttl": 300
}
```

### 🌐 Example: Multiple Zones

```json
{
  "cloudflare": [
    {
      "authentication": { "api_token": "your-api-token" },
      "zone_id": "first_zone_id",
      "subdomains": [
        { "name": "", "proxied": false }
      ]
    },
    {
      "authentication": { "api_token": "your-api-token" },
      "zone_id": "second_zone_id",
      "subdomains": [
        { "name": "", "proxied": false }
      ]
    }
  ],
  "a": true,
  "aaaa": true,
  "purgeUnknownRecords": false
}
```

### 🐳 Docker Compose (legacy config file)

```yml
version: '3.9'
services:
  cloudflare-ddns:
    image: ghcr.io/joenorth/cloudflare-ddns:latest
    container_name: cloudflare-ddns
    security_opt:
      - no-new-privileges:true
    network_mode: 'host'
    volumes:
      - /YOUR/PATH/HERE/config.json:/config.json
    restart: unless-stopped
```

### 🏁 Legacy CLI Flags

In legacy config mode, use `--repeat` to run continuously (the TTL value is used as the update interval):

```bash
cloudflare-ddns --repeat
cloudflare-ddns --repeat --dry-run
```

---

## 🔗 Helpful Links

- 🔑 [Cloudflare API token](https://dash.cloudflare.com/profile/api-tokens)
- 🆔 [Cloudflare zone ID](https://support.cloudflare.com/hc/en-us/articles/200167836-Where-do-I-find-my-Cloudflare-IP-address-)
- 📋 [Cloudflare zone DNS record ID](https://support.cloudflare.com/hc/en-us/articles/360019093151-Managing-DNS-records-in-Cloudflare)
- 🖥️ [Technitium DNS Server](https://technitium.com/dns/)

## 📜 License

This project is licensed under the GNU General Public License, version 3 (GPLv3).

## 👨‍💻 Original Author

Timothy Miller

[View their GitHub profile 💡](https://github.com/timothymiller)

[View their personal website 💻](https://itstmillertime.com)
