# 🏢 Office Electricity Monitor - IUT Techathon 2026

**🟢 Live Dashboard Demo:** [https://iut-techathon-2026.vercel.app/](https://iut-techathon-2026.vercel.app/)  
**⚙️ Live Backend API:** [https://iuttechathon2026-production.up.railway.app/health](https://iuttechathon2026-production.up.railway.app/health)  
**📄 Full Technical Report:** [report.tex](./report.tex) (Compile to PDF for the full 5-page architectural breakdown)

![Dashboard Overview](assets/image1.png)
![Time Travel Alerts](assets/image2_demo_alerts.png)
![System Diagram](system_diagram-1.png)

Welcome to the ultimate solution for the "Lights, Fans, Discord" problem. This project provides a highly robust backend API, a real-time React web dashboard, an interactive AI-powered Discord Bot, and a conceptual hardware schematic.

---

## 🏆 Hackathon Deliverables Checklist
We have meticulously fulfilled every requirement and bonus point in the rulebook:
- [x] **High-Level System Diagram:** Included in repo as `system_diagram.pdf`.
- [x] **Hardware Schematic:** Included in the `esp32_hardware_design/` folder (Conceptual ESP32 layout).
- [x] **Simulated Device Data:** Background Rust Simulator securely toggles 15 devices every 30s.
- [x] **Real-time Web Dashboard:** React UI polls `GET /api/devices` every 30s for live visual updates (no refresh needed).
- [x] **Discord Bot (`!status`, `!usage`, `!room`):** Built into the backend using `serenity`.
- [x] **AI Humanized Responses:** Integrates `rig` (supporting Gemini/OpenAI/Claude) to format raw JSON into conversational messages for the boss.
- [x] **Data Export Pipeline:** Typing `!export` queries the database, dynamically builds a CSV, uploads it to an S3/MinIO bucket, and returns a secure presigned download link.
- [x] **Bonus (Visual Layout):** Interactive 2D Floor Plan with glowing animations.
- [x] **Bonus (Proactive Alerts):** Discord Bot pushes autonomous alerts for After-Hours & 2-Hour Waste rules directly to `#office-alerts`.

---

## 🤖 Discord Bot Commands
You can interact with the office system directly through Discord. The bot supports natural language humanization for responses.
- `!status` - Get an overview of all active devices in the building.
- `!usage` - Check the live total wattage and daily kWh estimate.
- `!export` - Generates a CSV history report on AWS S3 and returns a download link.
- `!room <room_name>` - Get detailed device status for a specific room. 
  - **Allowed Room Aliases:**
    - Drawing Room: `drawing`, `drawingroom`
    - Work Room 1: `work1`, `workroom1`, `wr1`
    - Work Room 2: `work2`, `workroom2`, `wr2`
  - *Example:* `!room work1`

---

## 🧑‍⚖️ Note to Judges: How to Test the Anomalies
Because the rules require triggering alerts for **"Devices left on over 2 hours"** and **"Devices on after 5 PM"**, we built a **Time-Travel Debugger** directly into the frontend (bottom-left corner) so you don't have to wait to grade our project.

**How to test the 2-Hour Rule:**
1. Go to the [Live Dashboard](https://iut-techathon-2026.vercel.app/).
2. Turn **ON** all devices in a single room (e.g., Work Room 1).
3. Click the **+2 Hours** button in the Time Travel panel. 
4. *Result:* The backend instantly fast-forwards its clock by 2 hours. The dashboard will flash red, and the Discord bot will push a Critical Alert.

**How to test the Office Hours Rule (9 AM - 5 PM):**
1. Look at your current local time. The Discord bot relies on a state transition (from *Inside Office Hours* to *After Hours*) to trigger a push alert and avoid spamming.
2. If it is already past 5:00 PM in real life, first click **-2 Hours** or **-5 Hours** in the Time Travel panel to travel backward *into* the working day. Wait 5 seconds for the bot to clear its memory tracker.
3. Then, click **+2 Hours** or **+5 Hours** to push the clock past 5:00 PM (17:00) again to simulate closing time.
4. *Result:* As the system crosses the 5 PM boundary with devices still ON, it will instantly detect the After-Hours anomaly and trigger a fresh warning to both the dashboard and Discord bot!

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
3. **The Discord Bot (PUSH):** 🚨 **Judge Highlight:** The Discord bot is NOT a separate external script polling our API! It runs natively as a background thread *inside* the Rust Axum backend. Because it is built-in, it has direct memory access to the shared `AppState` cache (no HTTP requests needed). It silently monitors the data internally every 5 seconds, and if it detects an anomaly, it *proactively pushes* the AI-generated warning directly to Discord's servers.

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

## ⚙️ Backend Implementation Tasks & Code Highlights

Below is a detailed breakdown of the major backend tasks we accomplished, how we engineered them, and the actual code powering them.

### Task 1: Background Device Simulator
**What we did:** We needed a way to simulate a live office environment where devices turn on and off automatically without manual intervention.
**How we did it:** We spawned a detached asynchronous `tokio` task on server startup. Every 30 to 60 seconds, it randomly selects a device, queries its current status from PostgreSQL using a row-level lock (`FOR UPDATE`), toggles the status, logs the change to the `device_history` table, and syncs the in-memory cache.
**Code:**
```rust
pub fn start_simulator(
    pool: PgPool,
    memory_devices: Arc<RwLock<HashMap<String, Device>>>,
    memory_history: Arc<RwLock<Vec<DeviceHistory>>>,
) {
    tokio::spawn(async move {
        info!("Starting background device simulator...");
        loop {
            let seconds = rand::random::<u64>() % 30 + 30;
            sleep(Duration::from_secs(seconds)).await;

            if let Err(e) = toggle_random_device(&pool, &memory_devices, &memory_history).await {
                error!("Simulator error: {}", e);
            }
        }
    });
}
```

### Task 2: AI-Powered Discord Bot & Proactive Alerts (Push Mechanism)
**What we did:** We built a Discord bot that listens to commands (`!status`, `!usage`) and proactively sends warnings to an `#office-alerts` channel if anomalies are detected (e.g., devices left on after 5 PM, or on for over 2 hours).
**How we did it:** The bot runs natively inside the Axum server using the `serenity` crate. It polls the application state every 5 seconds without making HTTP requests. It uses the `rig` library to pass raw JSON data to an LLM (Gemini, OpenAI, etc.) to humanize the response before sending it to the boss.
**Code:**
```rust
async fn check_and_send_alerts(
    state: &AppState,
    channel_id: ChannelId,
    http: &Arc<serenity::http::Http>,
    sent_alerts: &Arc<Mutex<HashSet<String>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = &state.pool;
    // Fast-forward time support for judging
    let offset_hours = state.demo_time_offset.load(std::sync::atomic::Ordering::Relaxed);
    let now = Utc::now() + chrono::Duration::hours(offset_hours);
    
    // ... [Database Fetch Logic] ...

    let hour = now_dhaka.hour();
    let is_after_hours = hour < 9 || hour >= 17;

    if is_after_hours {
        // Find active devices and push an alert
        if !active_devices.is_empty() {
            let alert_msg = format!(
                "⚠️ **After-Hours Power Alert!**\nThe following devices are still active at {:02}:{:02} Dhaka time:\n{}\nDid someone forget to turn them off?",
                hour, now_dhaka.minute(), list
            );
            
            let humanized = humanize_response(state, &alert_msg).await;
            channel_id.say(http, humanized).await?;
        }
    }
    // ... [2-Hour Continuous Use Check Logic] ...
    Ok(())
}
```

### Task 3: Data Export Pipeline to S3
**What we did:** We implemented a data export feature allowing users to download the historical power usage of the office as a CSV file.
**How we did it:** When the `!export` command is triggered (or via API), the backend queries the `device_history` table, dynamically generates a CSV string in memory, uploads the byte stream directly to an S3/MinIO bucket using the `aws-sdk-s3` crate, and generates a pre-signed URL valid for 15 minutes. The URL is then cached in Redis.
**Code:**
```rust
async fn handle_export(state: &AppState) -> Result<String, Box<dyn std::error::Error>> {
    // ... [Query Database & Build CSV String] ...

    let file_key = format!("reports/power_report_{}.csv", Utc::now().timestamp());

    state.s3_client
        .put_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .body(aws_sdk_s3::primitives::ByteStream::from(csv_content.into_bytes()))
        .content_type("text/csv")
        .send()
        .await?;

    let presigned = state.s3_client
        .get_object()
        .bucket(&state.s3_bucket)
        .key(&file_key)
        .presigned(aws_sdk_s3::presigning::PresigningConfig::expires_in(Duration::from_secs(900))?)
        .await?;

    Ok(presigned.uri().to_string())
}
```

### Task 4: Fail-Safe Degradation and In-Memory Fallback
**What we did:** Ensuring 100% uptime even if the main PostgreSQL database goes down.
**How we did it:** When the application starts, it loads the initial database state into a thread-safe `tokio::sync::RwLock` cache. When an endpoint queries data, it first attempts to hit the database. If the database connection fails, it catches the error and silently reads from the memory cache instead.
**Code:**
```rust
async fn get_raw_status(state: &AppState) -> Result<String, sqlx::Error> {
    let pool = &state.pool;
    let devices_result = sqlx::query_as::<_, Device>("SELECT * FROM devices")
        .fetch_all(pool)
        .await;

    let devices = match devices_result {
        Ok(devs) => devs,
        Err(e) => {
            error!("Database offline, bot using in-memory cache for status: {}", e);
            let cache = state.memory_devices.read().await;
            cache.values().cloned().collect() // Fallback to memory
        }
    };
    
    // ... [Formatting Logic] ...
}
```

### Task 5: Time-Travel Debugging API
**What we did:** Created a way for judges to test time-based rules (like after 5 PM or continuous 2-hour use) without actually waiting.
**How we did it:** We store a global `demo_time_offset` using `std::sync::atomic::AtomicI64`. When the frontend sends a POST request to `/api/alerts/demo-time`, we safely update this offset across all threads.
**Code:**
```rust
#[derive(serde::Deserialize)]
pub struct DemoTimeRequest {
    pub offset_hours: i64,
}

async fn set_demo_time(
    State(state): State<AppState>,
    Json(payload): Json<DemoTimeRequest>,
) -> Result<StatusCode, StatusCode> {
    // Safely update the global time offset across all threads
    state.demo_time_offset.store(payload.offset_hours, Ordering::Relaxed);
    Ok(StatusCode::OK)
}
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
