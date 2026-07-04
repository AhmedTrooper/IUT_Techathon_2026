# 🏢 Office Electricity Monitor - IUT Techathon 2026

**🟢 Live Dashboard Demo:** [https://iut-techathon-2026.vercel.app/](https://iut-techathon-2026.vercel.app/)  
**📄 Full Technical Report:** [report.tex](./report.tex) (Compile to PDF for the full 5-page architectural breakdown)

![Dashboard Overview](assets/image1.png)
![Time Travel Alerts](assets/image2_demo_alerts.png)
![System Diagram](system_diagram-1.png)

Welcome to the ultimate solution for the "Lights, Fans, Discord" problem. This project provides a highly robust backend API, a real-time React web dashboard, an interactive AI-powered Discord Bot, and a conceptual hardware schematic.

---

## 🏆 Hackathon Deliverables Checklist
We have meticulously fulfilled every requirement and bonus point in the rulebook:
- [x] **High-Level System Diagram:** Included in repo as `system_diagram.pdf`.
- [x] **Hardware Schematic:** Included in the `wokwi/` folder (Conceptual ESP32 layout).
- [x] **Simulated Device Data:** Background Rust Simulator securely toggles 15 devices every 30s.
- [x] **Real-time Web Dashboard:** React UI polls `GET /api/devices` every 30s for live visual updates (no refresh needed).
- [x] **Discord Bot (`!status`, `!usage`, `!room`):** Built into the backend using `serenity`.
- [x] **AI Humanized Responses:** Integrates `rig` (supporting Gemini/OpenAI/Claude) to format raw JSON into conversational messages for the boss.
- [x] **Data Export Pipeline:** Typing `!export` queries the database, dynamically builds a CSV, uploads it to an S3/MinIO bucket, and returns a secure presigned download link.
- [x] **Bonus (Visual Layout):** Interactive 2D Floor Plan with glowing animations.
- [x] **Bonus (Proactive Alerts):** Discord Bot pushes autonomous alerts for After-Hours & 2-Hour Waste rules directly to `#office-alerts`.

---

## 🧑‍⚖️ Note to Judges: How to Test the Anomalies
Because the rules require triggering alerts for **"Devices left on over 2 hours"** and **"Devices on after 5 PM"**, we built a **Time-Travel Debugger** directly into the frontend (bottom-left corner) so you don't have to wait to grade our project.

**How to test the 2-Hour Rule:**
1. Go to the [Live Dashboard](https://iut-techathon-2026.vercel.app/).
2. Turn **ON** all devices in a single room (e.g., Work Room 1).
3. Click the **+2 Hours** button in the Time Travel panel. 
4. *Result:* The backend instantly fast-forwards its clock by 2 hours. The dashboard will flash red, and the Discord bot will push a Critical Alert.

**How to test the Office Hours Rule (9 AM - 5 PM):**
1. Look at your current local time. 
2. Use the Time Travel buttons to push the clock past 5:00 PM (17:00). *(e.g., If it is 1:00 PM, click **+5 Hours** to simulate 6:00 PM).*
3. *Result:* If any device is ON, the system will instantly detect the After-Hours anomaly and trigger a warning.

*Click "Reset Time" to snap the server back to real-world time.*

---

## 🛠️ Tech Stack
**Frontend:**
- **React + Vite:** Lightning-fast UI rendering.
- **Tailwind CSS:** Custom animations, glowing states, and responsive design.
- **Axios & TanStack:** Robust API polling and state management.

**Backend:**
- **Rust (Axum):** Extremely fast, memory-safe API routing.
- **PostgreSQL (SQLx):** Persistent, transactional source of truth for device states.
- **Redis:** High-speed caching for rate-limiting and daily kWh calculations.
- **Serenity & Rig:** WebSocket Discord integration and LLM prompt chaining.
- **AWS SDK (S3):** Real-time CSV generation and presigned URL hosting via MinIO S3 object storage.

---

## 🧠 System Architecture (Push vs. Pull)
Our system uses a strict **Single Source of Truth** architecture:
1. **The Core:** Every device toggle locks a PostgreSQL row, updates the status, and records a timestamp. 
2. **The Web Dashboard (PULL):** The React UI uses an HTTP polling mechanism (`GET /api/devices`) every 30 seconds to *pull* the latest data and map it visually.
3. **The Discord Bot (PUSH):** The bot runs natively *inside* the Rust API. It has direct memory access to the database cache (no HTTP requests needed). It checks the data internally every 5 seconds, and if it detects an anomaly, it *pushes* a warning to Discord's servers.

---

## 📊 Data Model
Our PostgreSQL database is designed for speed and historical tracking:

```sql
-- The Current Live State
CREATE TABLE devices (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    room VARCHAR(50) NOT NULL,       -- e.g., 'work_room_1'
    device_type VARCHAR(20) NOT NULL, -- 'fan' or 'light'
    status BOOLEAN NOT NULL DEFAULT false,
    power_consumption INTEGER NOT NULL, -- Wattage
    last_changed TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Immutable Append-Only Ledger for Power Analytics
CREATE TABLE device_history (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(50) REFERENCES devices(id),
    status BOOLEAN NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

## ⚡ Fail-Safe Degradation
If the PostgreSQL database or Redis cache crashes during the demo, the backend instantly falls back to an internal `tokio::RwLock` memory cache. **The dashboard and bot will never go offline.** The API intercepts the SQL connection failure, serves the memory cache, and logs the degradation seamlessly.

---

## 🧪 API Endpoints & Automated Testing

### REST Endpoints
The backend provides a fully permissive CORS policy, allowing any client to interface with the office state:
- `GET /health`: Returns health status of Postgres, Redis, and S3 dependencies.
- `GET /api/devices`: Returns the live status of all 15 devices.
- `POST /api/devices/{id}/toggle`: Safely toggles a device state with row-level locking.
- `GET /api/usage`: Returns the live power meter and today's total estimated kWh.
- `GET /api/alerts`: Returns active anomalies (after-hours usage, 2-hour continuous waste).
- `POST /api/alerts/demo-time`: Fast-forwards the server clock to test time-based rules.
- `POST /api/reports/export`: Dynamically builds a CSV of device history, uploads to S3, and returns a presigned download link.

### Test Suites
This repository includes comprehensive automated testing for both the frontend and backend architectures:

**1. Backend Graceful Degradation Tests (Rust)**
We aggressively test our Fail-Safe mechanisms by forcefully injecting broken/fake database connections into the Axum router and strictly verifying that every critical endpoint intercepts the failure and returns accurate data from the in-memory cache.
```bash
cd api
cargo test
```

**2. Frontend Unit Tests (React/Vitest)**
The React dashboard includes mocked unit tests (using Vitest and React Testing Library) to validate the Time Travel UI and rendering logic.
```bash
cd web
bunx vitest run
```

---

## 🚀 Local Development Setup

We have provided a fully containerized setup using Docker and Makefiles for an effortless developer experience.

### 1. Environment Variables
Create a `.env` file in the root directory (see `.env.example`):
```env
DATABASE_URL=postgresql://user:pass@localhost:5432/iut_techathon_2026_db
REDIS_URL=redis://localhost:6379
DISCORD_TOKEN=your_bot_token
DISCORD_ALERT_CHANNEL_ID=your_channel_id
AI_PROVIDER=GEMINI
AI_MODEL=gemini-1.5-flash
AI_API_KEY=your_gemini_key
```

### 2. Start Services (Docker)
Ensure Docker is running, then use the Makefile to spin up Postgres and Redis:
```bash
make docker
```
*(To view logs: `make docker-logs` | To tear down: `make docker-down`)*

### 3. Run the Backend (Rust)
You can run the backend manually via Cargo, or use the provided Makefile shortcut:
```bash
make api
```
*The API will be available at `http://localhost:8080`.*

### 4. Run the Frontend (React)
Install the dependencies first, then start the Vite server using Make:
```bash
cd web && bun install
cd ..
make frontend
```
*The UI will be available at `http://localhost:5173`.*
