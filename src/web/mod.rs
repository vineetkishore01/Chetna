//! Web Dashboard - Production-grade UI for memory visualization and management
//! Minimalist OLED black theme with reactive onboarding

use axum::{
    routing::get,
    Router,
    response::Html,
};
use std::sync::Arc;
use crate::{Brain, config_file::UserConfig};

pub fn routes() -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(dashboard))
        .route("/memories", get(memories_page))
        .route("/sessions", get(sessions_page))
        .route("/settings", get(settings_page))
}

const STYLE: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, "Inter", "Segoe UI", Roboto, sans-serif;
    background: #000000;
    color: #ffffff;
    min-height: 100vh;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
}
.header {
    background: rgba(10, 10, 10, 0.8);
    backdrop-filter: blur(10px);
    padding: 0.75rem 1.5rem;
    border-bottom: 1px solid #222;
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: sticky;
    top: 0;
    z-index: 100;
}
.logo-area { display: flex; align-items: center; gap: 1rem; }
.logo-area h1 { font-size: 1rem; font-weight: 700; letter-spacing: -0.02em; }
.status-pill {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: #111;
    padding: 0.25rem 0.75rem;
    border-radius: 100px;
    font-size: 0.75rem;
    color: #888;
    border: 1px solid #222;
}
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: #444; }
.status-dot.online { background: #10b981; box-shadow: 0 0 8px #10b981; }
.status-dot.offline { background: #ef4444; }

.nav { display: flex; gap: 0.25rem; }
.nav a {
    color: #888;
    text-decoration: none;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    transition: all 0.2s;
}
.nav a:hover, .nav a.active { color: #fff; background: #1a1a1a; }

.container { padding: 2rem 1.5rem; max-width: 1000px; margin: 0 auto; }

/* Onboarding Styles */
.onboarding-card {
    max-width: 500px;
    margin: 4rem auto;
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 12px;
    padding: 2.5rem;
    box-shadow: 0 20px 40px rgba(0,0,0,0.4);
}
.onboarding-card h2 { font-size: 1.5rem; margin-bottom: 0.5rem; }
.onboarding-card p { color: #888; font-size: 0.9rem; margin-bottom: 2rem; }

.setup-step { display: none; }
.setup-step.active { display: block; animation: fadeIn 0.4s ease; }

@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

/* Command Center Styles */
.command-bar {
    background: #0a0a0a;
    border: 1px solid #333;
    border-radius: 12px;
    padding: 1rem 1.5rem;
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
    transition: border-color 0.2s;
    box-shadow: 0 4px 20px rgba(0,0,0,0.2);
}
.command-bar:focus-within { border-color: #555; }
.command-input {
    background: transparent;
    border: none;
    color: #fff;
    font-size: 1.1rem;
    width: 100%;
    outline: none;
}
.command-input::placeholder { color: #444; }

.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; }
.stat-box {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 10px;
    padding: 1.25rem;
}
.stat-box .label { font-size: 0.75rem; text-transform: uppercase; color: #555; font-weight: 600; margin-bottom: 0.5rem; }
.stat-box .value { font-size: 1.75rem; font-weight: 700; color: #fff; }

.btn {
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
}
.btn-primary { background: #fff; color: #000; }
.btn-primary:hover { background: #ccc; }
.btn-outline { background: transparent; border-color: #333; color: #888; }
.btn-outline:hover { border-color: #555; color: #fff; }

input[type="text"], input[type="password"], select {
    background: #111;
    border: 1px solid #222;
    padding: 0.75rem;
    border-radius: 8px;
    color: #fff;
    margin-bottom: 1rem;
}
input:focus { border-color: #444; outline: none; }

.toast {
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    background: #fff;
    color: #000;
    padding: 0.75rem 1.5rem;
    border-radius: 8px;
    font-weight: 600;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    display: none;
    z-index: 1000;
}
"#;

async fn dashboard() -> Html<String> {
    Html(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chetna | Command Center</title>
    <style>{}</style>
</head>
<body>
    <header class="header">
        <div class="logo-area">
            <h1>CHETNA</h1>
            <div class="status-pill">
                <div id="statusDot" class="status-dot"></div>
                <span id="statusText">Checking...</span>
            </div>
        </div>
        <nav class="nav">
            <a href="/" class="active">Dashboard</a>
            <a href="/memories">Memories</a>
            <a href="/sessions">Sessions</a>
            <a href="/settings">Settings</a>
        </nav>
    </header>

    <main class="container" id="mainContent">
        <div id="loadingView" class="loading">Initializing Brain...</div>
        
        <!-- Onboarding View -->
        <div id="onboardingView" style="display: none;">
            <div class="onboarding-card">
                <div id="step1" class="setup-step active">
                    <h2>Welcome to Chetna</h2>
                    <p>Let's configure your memory engine. Where should we get your embeddings?</p>
                    <label>Provider</label>
                    <select id="setupProvider">
                        <option value="ollama">Ollama (Local)</option>
                        <option value="openai">OpenAI (Cloud)</option>
                    </select>
                    <div id="ollamaFields">
                        <label>Base URL</label>
                        <input type="text" id="setupBaseUrl" value="http://localhost:11434">
                    </div>
                    <div id="openaiFields" style="display:none;">
                        <label>API Key</label>
                        <input type="password" id="setupApiKey" placeholder="sk-...">
                    </div>
                    <button class="btn btn-primary" onclick="testAndNext()">Next: Validate Connection</button>
                </div>
                
                <div id="step2" class="setup-step">
                    <h2>Connecting...</h2>
                    <p id="validationMsg">Waiting for response from provider...</p>
                    <div id="modelSelectArea" style="display:none;">
                        <label>Select Model</label>
                        <select id="setupModel"></select>
                        <button class="btn btn-primary" onclick="completeSetup()">Finish Setup</button>
                    </div>
                    <button id="retryBtn" class="btn btn-outline" style="display:none;" onclick="location.reload()">Retry</button>
                </div>
            </div>
        </div>

        <!-- Dashboard View -->
        <div id="dashboardView" style="display: none;">
            <div class="command-bar">
                <span style="color: #555;">🔍</span>
                <input type="text" class="command-input" id="mainSearch" placeholder="Search across all memories..." onkeyup="handleSearch(event)">
            </div>

            <div class="grid">
                <div class="stat-box">
                    <div class="label">Total Memories</div>
                    <div class="value" id="countMemories">0</div>
                </div>
                <div class="stat-box">
                    <div class="label">Active Recall Events</div>
                    <div class="value" id="countRecall">0</div>
                </div>
                <div class="stat-box">
                    <div class="label">Skills & Procedures</div>
                    <div class="value" id="countSkills">0</div>
                </div>
            </div>

            <div id="searchResults" style="margin-top: 2rem;"></div>
        </div>
    </main>

    <div id="toast" class="toast"></div>

    <script>
        let config = null;

        async fn init() {{
            const resp = await fetch('/api/config');
            config = await resp.json();
            
            // Check if configured
            if (!config.provider || (config.provider === 'ollama' && !config.base_url)) {{
                showOnboarding();
            }} else {{
                showDashboard();
                checkHealth();
            }}
        }}

        function showOnboarding() {{
            document.getElementById('loadingView').style.display = 'none';
            document.getElementById('onboardingView').style.display = 'block';
        }}

        function showDashboard() {{
            document.getElementById('loadingView').style.display = 'none';
            document.getElementById('dashboardView').style.display = 'block';
            loadStats();
        }}

        async function checkHealth() {{
            try {{
                const resp = await fetch('/api/config/health');
                const data = await resp.json();
                const dot = document.getElementById('statusDot');
                const text = document.getElementById('statusText');
                
                if (data.connected) {{
                    dot.className = 'status-dot online';
                    text.innerText = 'System Live';
                }} else {{
                    dot.className = 'status-dot offline';
                    text.innerText = 'Provider Disconnected';
                }}
            }} catch (e) {{
                console.error(e);
            }}
        }}

        async function testAndNext() {{
            const provider = document.getElementById('setupProvider').value;
            const baseUrl = document.getElementById('setupBaseUrl').value;
            const apiKey = document.getElementById('setupApiKey').value;

            document.getElementById('step1').classList.remove('active');
            document.getElementById('step2').classList.add('active');

            try {{
                const resp = await fetch('/api/config/ping', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ provider, base_url: baseUrl, api_key: apiKey, model: 'test' }})
                }});
                const data = await resp.json();
                
                if (data.success) {{
                    document.getElementById('validationMsg').innerText = '✓ Connection Successful';
                    // In a real app, we'd fetch models here
                    document.getElementById('modelSelectArea').style.display = 'block';
                    const modelSelect = document.getElementById('setupModel');
                    const models = provider === 'ollama' ? ['nomic-embed-text', 'mxbai-embed-large'] : ['text-embedding-3-small'];
                    models.forEach(m => {{
                        const opt = document.createElement('option');
                        opt.value = m; opt.innerText = m;
                        modelSelect.appendChild(opt);
                    }});
                }} else {{
                    document.getElementById('validationMsg').innerText = '✕ Connection Failed: ' + data.message;
                    document.getElementById('retryBtn').style.display = 'block';
                }}
            }} catch (e) {{
                document.getElementById('validationMsg').innerText = '✕ Error: ' + e.message;
                document.getElementById('retryBtn').style.display = 'block';
            }}
        }}

        async function completeSetup() {{
            const provider = document.getElementById('setupProvider').value;
            const baseUrl = document.getElementById('setupBaseUrl').value;
            const apiKey = document.getElementById('setupApiKey').value;
            const model = document.getElementById('setupModel').value;

            const resp = await fetch('/api/config', {{
                method: 'POST',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ provider, base_url: baseUrl, api_key: apiKey, model }})
            }});
            
            if (resp.ok) {{
                showToast('Brain Initialized');
                location.reload();
            }}
        }}

        async function loadStats() {{
            const resp = await fetch('/api/stats');
            const data = await resp.json();
            document.getElementById('countMemories').innerText = data.total_memories || 0;
            document.getElementById('countRecall').innerText = data.total_sessions || 0;
            document.getElementById('countSkills').innerText = (data.total_skills || 0) + (data.total_procedures || 0);
        }}

        function showToast(msg) {{
            const t = document.getElementById('toast');
            t.innerText = msg;
            t.style.display = 'block';
            setTimeout(() => t.style.display = 'none', 3000);
        }}

        init();
    </script>
</body>
</html>
"#, STYLE))
}

async fn memories_page() -> Html<String> {
    Html("<h1>Memories Page</h1>".to_string())
}

async fn sessions_page() -> Html<String> {
    Html("<h1>Sessions Page</h1>".to_string())
}

async fn settings_page() -> Html<String> {
    Html("<h1>Settings Page</h1>".to_string())
}
