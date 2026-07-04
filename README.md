# IUT Techathon 2026 - Office Monitoring System

## Overview
This is the complete backend and hardware simulation suite for the Boss's "Lights, Fans, Discord" office monitoring system. It provides a highly robust, fail-safe backend API, an interactive Discord Bot, and live data simulation for 15 office devices across 3 rooms.

## System Architecture
The system consists of three main components:
1. **Simulated Hardware (Wokwi):** A representative ESP32 schematic demonstrating how the physical sensors (relays/switches) are wired and read.
2. **Rust Backend API (Axum):** The central source of truth. It manages the PostgreSQL database, Redis caching, AWS S3 report generation, and provides REST endpoints.
3. **Discord Bot (Serenity & LLM):** A chat interface that dynamically translates live API data into humanized responses using AI (Gemini/OpenAI) and sends proactive anomaly alerts.

*(A high-level system diagram is included in this repository as `system_diagram.pdf`)*

## Setup Instructions

### 1. Prerequisites
- **Rust Toolchain:** (v1.75+) `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **PostgreSQL:** A running Postgres instance.
- **Redis:** A running Redis server for rate-limiting and query caching.

### 2. Environment Variables
Create a `.env` file in the `api/` directory with the following keys:
```env
# Database & Cache
DATABASE_URL=postgres://user:pass@localhost:5432/techathon
REDIS_URL=redis://127.0.0.1/

# Discord Bot
DISCORD_TOKEN=your_discord_bot_token
DISCORD_ALERT_CHANNEL_ID=your_discord_channel_id_for_alerts

# AI Humanization (Choose One)
GEMINI_API_KEY=your_gemini_key
# OPENAI_API_KEY=your_openai_key

# AWS S3 (For Reports & Diagram Storage)
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1
S3_BUCKET_NAME=your_bucket_name
```

### 3. Database Initialization
You must create the PostgreSQL tables before running the server:
```sql
CREATE TABLE devices (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    room VARCHAR(50) NOT NULL,
    device_type VARCHAR(20) NOT NULL,
    status BOOLEAN NOT NULL DEFAULT false,
    power_consumption INTEGER NOT NULL,
    last_changed TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE device_history (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(50) REFERENCES devices(id),
    status BOOLEAN NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```
*(Ensure you seed the 15 devices into the `devices` table initially).*

### 4. Running the Backend
Navigate to the `api` directory and run:
```bash
cd api
cargo run --release
```
The backend will automatically:
1. Start the REST API on `http://0.0.0.0:8080`.
2. Boot the Serenity Discord Bot.
3. Spin up the Background Simulator (toggling devices every 30-60s).
4. Launch the Proactive Alerting Engine (checking for wasted power every 60s).

## Features & Endpoints
- `GET /api/devices`: Live status of all 15 devices.
- `POST /api/devices/{id}/toggle`: Mutate a device state (with DB row locking).
- `GET /api/usage`: Live power meter and today's total estimated kWh (Redis cached).
- `GET /api/alerts`: Active anomaly detection (after-hours usage, 2-hour continuous waste).
- `POST /api/reports/export`: Generate a CSV report and upload to S3.

## Hardware Schematic
Inside the `wokwi/` directory, you will find `diagram.json` and `sketch.ino`. 
Upload these to [Wokwi](https://wokwi.com) to view the representative circuit for a single room, demonstrating how an ESP32 reads 5 slide switches and operates 3 LEDs (Lights) and 2 DC Motors (Fans).

## Fail-Safe Design
The backend employs a strict "Graceful Degradation" protocol. If the PostgreSQL database crashes or disconnects during the demo, the API and Discord Bot will seamlessly fall back to an internal `tokio::sync::RwLock` memory cache. The dashboard will never go down.

## Robust Automated Testing
The API is fully covered by robust automated tests located in `api/src/tests.rs`. 
These tests specifically verify our Graceful Degradation systems. They forcefully inject broken/fake database connections and offline Redis configurations into the `axum` router, and then strictly verify that every critical endpoint (`/devices`, `/usage`, `/alerts`) successfully intercepts the failure and returns accurate data from the in-memory cache.

To execute the test suite, run:
```bash
cd api
cargo test
```
