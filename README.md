# 🏢 Office Electricity Monitor - IUT Techathon 2026

## 🌟 Overview
This is the complete, full-stack software and hardware simulation suite for the "Lights, Fans, Discord" office monitoring system. It provides a highly robust backend API, a real-time web dashboard, an interactive Discord Bot powered by AI, and a conceptual hardware schematic. 

The entire system is strictly built to satisfy every rule, requirement, and bonus point of the Hackathon.

## 🏗️ System Architecture & Consistency
The system was designed with a strict **Single Source of Truth** architecture to ensure that the Discord bot and the Web Dashboard always show the exact same reality.

1. **The Core (Rust Backend & PostgreSQL):** The Axum API manages the PostgreSQL database. Every time a device state changes, it is locked, updated, and timestamped in Postgres.
2. **The Hardware Simulator (Background Task):** A background thread safely toggles random devices every 30-60 seconds to simulate a live office. 
3. **The Web Dashboard (React/Vite):** A beautiful, responsive UI that polls the backend every 30 seconds for live data. It maps device data onto a 2D floor plan of the office.
4. **The Discord Bot (Serenity & Rig):** A bot that connects to the same backend. It queries the same database as the dashboard, feeds that raw data into an LLM (Gemini/OpenAI/Claude), and posts humanized responses back to Discord.
5. **The Proactive Alerts Engine:** Both the dashboard and the Discord bot share a unified alerts engine that constantly checks for anomalies (e.g., 2-hour waste, after-hours usage).

*(A high-level visual system diagram is included in this repository as `system_diagram.pdf`)*

## 🚀 Key Features
*   **Fail-Safe Degradation:** If the PostgreSQL database or Redis cache crashes, the entire backend instantly falls back to an internal `tokio::RwLock` memory cache. The dashboard and bot will never go offline.
*   **Time-Travel Debugging:** A built-in UI feature that allows judges to fast-forward the server's internal clock to instantly demonstrate time-based anomaly alerts (Office Hours rule, 2-Hour rule) without waiting.
*   **AI Humanization:** The Discord bot uses the `rig` crate to translate JSON device data into warm, friendly messages.
*   **Permissive CORS:** The API is accessible from any web frontend or automated testing system.

## 🛠️ Setup Instructions

### 1. Prerequisites
*   **Rust Toolchain:** (v1.75+)
*   **Node.js & Bun/npm:** For the frontend.
*   **PostgreSQL & Redis:** Running locally or via Docker.

### 2. Environment Variables
Create a `.env` file in the project root:
```env
# Database & Cache
DATABASE_URL=postgresql://user:pass@localhost:5432/iut_techathon_2026_db
REDIS_URL=redis://localhost:6379

# S3 Storage (For Reports)
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=admin
S3_SECRET_KEY=supersecretpassword
S3_BUCKET=iut-techathon-2026-bucket

# Discord Integration
DISCORD_TOKEN=your_discord_bot_token
DISCORD_ALERT_CHANNEL_ID=your_discord_channel_id

# AI Humanization (Options: GEMINI, ANTHROPIC, OPENAI, GROQ, OPENROUTER, XAI)
AI_PROVIDER=GEMINI
AI_MODEL=gemini-1.5-flash
AI_API_KEY=your_api_key
```

### 3. Running the Backend (API & Bot)
The Rust backend automatically seeds the database with the 15 required devices on startup.
```bash
cd api
cargo run --release
```
*This starts the API on `http://localhost:8080`, boots the Discord bot, and launches the hardware simulator.*

### 4. Running the Web Dashboard
```bash
cd web
npm install
npm run dev
```
*This starts the Vite React frontend on `http://localhost:5173`.*

## 🔌 Hardware Schematic (Wokwi)
Inside the `wokwi/` directory, you will find `diagram.json` and `sketch.ino`. 

As requested by the rulebook, this is a **representative conceptual circuit** for a single room. It demonstrates how an ESP32 microcontroller is wired to read 5 slide switches and operate 3 LEDs (Lights) and 2 DC Motors (Fans) using pins 15, 2, 4, 16, and 17. 

The `sketch.ino` C++ code reads these pins and formats a JSON payload. In a physical implementation, the ESP32 would simply send this JSON payload to our Rust API via an HTTP POST request. Because this is a simulation, our backend handles the live data generation internally.

## 📡 API Endpoints
*   `GET /api/devices`: Live status of all 15 devices.
*   `POST /api/devices/{id}/toggle`: Safely toggle a device state.
*   `GET /api/usage`: Live power meter and today's total estimated kWh.
*   `GET /api/alerts`: Active anomaly detection (after-hours usage, 2-hour continuous waste).
*   `POST /api/alerts/demo-time`: (Internal/Debug) Fast-forward the server clock to trigger rules.

## 🤖 Discord Bot Commands
*   `!status`: Shows the overall summary of all 3 rooms.
*   `!room <name>`: Shows detailed status for a specific room (e.g. `!room work1`).
*   `!usage`: Shows the current live wattage and daily kWh estimate.
*   `!export`: Generates a CSV report of device history.
