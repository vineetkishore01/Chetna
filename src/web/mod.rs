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

fn layout(content: &str, active_tab: &str) -> String {
    format!(r#"
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
            <a href="/" class="{}">Dashboard</a>
            <a href="/memories" class="{}">Memories</a>
            <a href="/sessions" class="{}">Sessions</a>
            <a href="/settings" class="{}">Settings</a>
        </nav>
    </header>

    <main class="container" id="mainContent">
        {}
    </main>

    <div id="toast" class="toast"></div>

    <script>
        async function checkHealth() {{
            try {{
                const resp = await fetch('/api/config/health');
                if (!resp.ok) throw new Error('Health check failed');
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
                const dot = document.getElementById('statusDot');
                const text = document.getElementById('statusText');
                dot.className = 'status-dot offline';
                text.innerText = 'API Offline';
                console.error(e);
            }}
        }}

        function showToast(msg) {{
            const t = document.getElementById('toast');
            t.innerText = msg;
            t.style.display = 'block';
            setTimeout(() => t.style.display = 'none', 3000);
        }}

        checkHealth();
        setInterval(checkHealth, 5000);
    </script>
</body>
</html>
"#, 
    STYLE,
    if active_tab == "dashboard" { "active" } else { "" },
    if active_tab == "memories" { "active" } else { "" },
    if active_tab == "sessions" { "active" } else { "" },
    if active_tab == "settings" { "active" } else { "" },
    content)
}

async fn dashboard() -> Html<String> {
    let dashboard_content = r#"
        <div id="loadingView" style="text-align: center; padding: 4rem;">Initializing Brain...</div>
        
        <!-- Onboarding View -->
        <div id="onboardingView" style="display: none;">
            <div class="onboarding-card">
                <div id="step1" class="setup-step active">
                    <h2>Welcome to Chetna</h2>
                    <p>Let's configure your memory engine. Where should we get your embeddings?</p>
                    <label>Provider</label>
                    <select id="setupProvider" style="width: 100%;" onchange="toggleProviderFields()">
                        <option value="ollama">Ollama (Local)</option>
                        <option value="openai">OpenAI (Cloud)</option>
                    </select>
                    <div id="ollamaFields">
                        <label>Base URL</label>
                        <input type="text" id="setupBaseUrl" value="http://localhost:11434" style="width: 100%;">
                    </div>
                    <div id="openaiFields" style="display:none;">
                        <label>API Key</label>
                        <input type="password" id="setupApiKey" placeholder="sk-..." style="width: 100%;">
                    </div>
                    <button class="btn btn-primary" onclick="testAndNext()">Next: Validate Connection</button>
                </div>
                
                <div id="step2" class="setup-step">
                    <h2>Connecting...</h2>
                    <p id="validationMsg">Waiting for response from provider...</p>
                    <div id="modelSelectArea" style="display:none;">
                        <label>Select Model</label>
                        <select id="setupModel" style="width: 100%;"></select>
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

        <script>
            let config = null;

            function toggleProviderFields() {
                const provider = document.getElementById('setupProvider').value;
                document.getElementById('ollamaFields').style.display = provider === 'ollama' ? 'block' : 'none';
                document.getElementById('openaiFields').style.display = provider === 'openai' ? 'block' : 'none';
            }

            async function init() {
                try {
                    const resp = await fetch('/api/config');
                    if (!resp.ok) throw new Error('Failed to load config');
                    config = await resp.json();
                    
                    if (!config.provider || (config.provider === 'ollama' && !config.base_url)) {
                        showOnboarding();
                    } else {
                        showDashboard();
                    }
                } catch (e) {
                    console.error("Init failed", e);
                    document.getElementById('loadingView').innerText = "✕ Failed to connect to Brain API";
                }
            }

            function showOnboarding() {
                document.getElementById('loadingView').style.display = 'none';
                document.getElementById('onboardingView').style.display = 'block';
            }

            function showDashboard() {
                document.getElementById('loadingView').style.display = 'none';
                document.getElementById('dashboardView').style.display = 'block';
                loadStats();
            }

            async function testAndNext() {
                const provider = document.getElementById('setupProvider').value;
                const baseUrl = document.getElementById('setupBaseUrl').value;
                const apiKey = document.getElementById('setupApiKey').value;

                document.getElementById('step1').classList.remove('active');
                document.getElementById('step2').classList.add('active');

                try {
                    const resp = await fetch('/api/config/ping', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ provider, base_url: baseUrl, api_key: apiKey, model: 'test' })
                    });
                    const data = await resp.json();
                    
                    if (data.success) {
                        document.getElementById('validationMsg').innerText = '✓ Connection Successful';
                        document.getElementById('modelSelectArea').style.display = 'block';
                        const modelSelect = document.getElementById('setupModel');
                        
                        // Try to get actual models from status endpoint
                        try {
                            // Temporary update config so status check works
                            await fetch('/api/config', {
                                method: 'POST',
                                headers: { 'Content-Type': 'application/json' },
                                body: JSON.stringify({ provider, base_url: baseUrl, api_key: apiKey, model: 'nomic-embed-text' })
                            });
                            
                            const statusResp = await fetch('/api/status/connections');
                            const statusData = await statusResp.json();
                            
                            if (statusData.embedding && statusData.embedding.available_models && statusData.embedding.available_models.length > 0) {
                                statusData.embedding.available_models.forEach(m => {
                                    const opt = document.createElement('option');
                                    opt.value = m; opt.innerText = m;
                                    modelSelect.appendChild(opt);
                                });
                            } else {
                                throw new Error('No models found');
                            }
                        } catch (err) {
                            console.warn("Could not fetch models, using defaults", err);
                            const models = provider === 'ollama' ? ['nomic-embed-text', 'mxbai-embed-large', 'qwen3-embedding:4b'] : ['text-embedding-3-small'];
                            models.forEach(m => {
                                const opt = document.createElement('option');
                                opt.value = m; opt.innerText = m;
                                modelSelect.appendChild(opt);
                            });
                        }
                    } else {
                        document.getElementById('validationMsg').innerText = '✕ Connection Failed: ' + data.message;
                        document.getElementById('retryBtn').style.display = 'block';
                    }
                } catch (e) {
                    document.getElementById('validationMsg').innerText = '✕ Error: ' + e.message;
                    document.getElementById('retryBtn').style.display = 'block';
                }
            }

            async function completeSetup() {
                const provider = document.getElementById('setupProvider').value;
                const baseUrl = document.getElementById('setupBaseUrl').value;
                const apiKey = document.getElementById('setupApiKey').value;
                const model = document.getElementById('setupModel').value;

                const resp = await fetch('/api/config', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ provider, model, base_url: baseUrl, api_key: apiKey })
                });
                
                if (resp.ok) {
                    showToast('Brain Initialized');
                    location.reload();
                }
            }

            async function loadStats() {
                try {
                    const resp = await fetch('/api/stats');
                    const data = await resp.json();
                    document.getElementById('countMemories').innerText = data.total_memories || 0;
                    document.getElementById('countRecall').innerText = data.total_sessions || 0;
                    document.getElementById('countSkills').innerText = (data.total_skills || 0) + (data.total_procedures || 0);
                } catch (e) {
                    console.error("Load stats failed", e);
                }
            }

            let searchTimeout = null;
            async function handleSearch(event) {
                const query = event.target.value;
                if (query.length < 3) {
                    document.getElementById('searchResults').innerHTML = '';
                    return;
                }
                
                if (searchTimeout) clearTimeout(searchTimeout);
                
                searchTimeout = setTimeout(async () => {
                    try {
                        const resp = await fetch(`/api/memory/search?query=${encodeURIComponent(query)}&limit=10`);
                        const memories = await resp.json();
                        
                        const container = document.getElementById('searchResults');
                        if (memories.length === 0) {
                            container.innerHTML = '<div style="color: #444; text-align: center; padding: 2rem;">No matching memories found.</div>';
                            return;
                        }
                        
                        container.innerHTML = `
                            <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Search Results</h3>
                            ${memories.map(m => `
                                <div style="background: #0a0a0a; padding: 1.25rem; border-radius: 10px; margin-bottom: 0.75rem; border: 1px solid #222; transition: border-color 0.2s;" onmouseover="this.style.borderColor='#444'" onmouseout="this.style.borderColor='#222'">
                                    <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem;">
                                        <span style="font-size: 0.65rem; background: #1a1a1a; color: #888; padding: 0.2rem 0.5rem; border-radius: 4px; text-transform: uppercase; letter-spacing: 0.05em;">${m.category}</span>
                                        <span style="font-size: 0.65rem; color: #444;">${new Date(m.created_at).toLocaleDateString()}</span>
                                    </div>
                                    <div style="font-size: 0.95rem; color: #eee; line-height: 1.5;">${m.content}</div>
                                    ${m.tags && m.tags.length > 0 ? `
                                        <div style="margin-top: 0.75rem; display: flex; gap: 0.4rem; flex-wrap: wrap;">
                                            ${m.tags.map(t => `<span style="font-size: 0.6rem; color: #555; border: 1px solid #222; padding: 0.1rem 0.4rem; border-radius: 3px;">#${t}</span>`).join('')}
                                        </div>
                                    ` : ''}
                                </div>
                            `).join('')}
                        `;
                    } catch (e) {
                        console.error("Search failed", e);
                    }
                }, 300);
            }

            init();
        </script>
    "#;
    
    Html(layout(dashboard_content, "dashboard"))
}

async fn memories_page() -> Html<String> {
    let content = r#"
        <h2>Memories</h2>
        <p style="color: #888; margin-bottom: 2rem;">Explore and manage your long-term memory graph.</p>
        <div id="memoriesList" class="grid"></div>
        
        <script>
            async function loadMemories() {
                try {
                    const resp = await fetch('/api/memory?limit=50');
                    const data = await resp.json();
                    const container = document.getElementById('memoriesList');
                    container.innerHTML = data.map(m => `
                        <div class="stat-box">
                            <div class="label">${m.category}</div>
                            <div class="value" style="font-size: 0.9rem; line-height: 1.4;">${m.content}</div>
                            <div style="margin-top: 1rem; font-size: 0.7rem; color: #444;">${new Date(m.created_at).toLocaleString()}</div>
                        </div>
                    `).join('');
                } catch (e) {
                    console.error("Load memories failed", e);
                }
            }
            loadMemories();
        </script>
    "#;
    Html(layout(content, "memories"))
}

async fn sessions_page() -> Html<String> {
    let content = r#"
        <h2>Sessions</h2>
        <p style="color: #888; margin-bottom: 2rem;">Active thought streams and conversation history.</p>
        <div id="sessionsList" class="grid"></div>
        
        <script>
            async function loadSessions() {
                try {
                    const resp = await fetch('/api/session');
                    const data = await resp.json();
                    const container = document.getElementById('sessionsList');
                    container.innerHTML = data.map(s => `
                        <div class="stat-box">
                            <div class="label">Session ${s.id.substring(0,8)}</div>
                            <div class="value" style="font-size: 0.9rem;">${s.summary || 'Active session'}</div>
                        </div>
                    `).join('');
                } catch (e) {
                    console.error("Load sessions failed", e);
                }
            }
            loadSessions();
        </script>
    "#;
    Html(layout(content, "sessions"))
}

async fn settings_page() -> Html<String> {
    let content = r#"
        <h2>Settings</h2>
        <p style="color: #888; margin-bottom: 2rem;">Engine configuration and provider management.</p>
        <div class="stat-box" style="max-width: 600px;">
            <div id="settingsForm"></div>
            <button class="btn btn-primary" onclick="saveSettings()">Save Changes</button>
        </div>
        
        <script>
            async function loadSettings() {
                try {
                    const resp = await fetch('/api/config');
                    const data = await resp.json();
                    const container = document.getElementById('settingsForm');
                    container.innerHTML = `
                        <div style="margin-bottom: 1.5rem;">
                            <label class="label">Provider</label>
                            <select id="editProvider" style="width: 100%;">
                                <option value="ollama" ${data.provider === 'ollama' ? 'selected' : ''}>Ollama</option>
                                <option value="openai" ${data.provider === 'openai' ? 'selected' : ''}>OpenAI</option>
                            </select>
                        </div>
                        <div style="margin-bottom: 1.5rem;">
                            <label class="label">Base URL / API Endpoint</label>
                            <input type="text" id="editBaseUrl" value="${data.base_url || ''}" style="width: 100%;">
                        </div>
                        <div style="margin-bottom: 1.5rem;">
                            <label class="label">Model</label>
                            <input type="text" id="editModel" value="${data.model || ''}" style="width: 100%;">
                        </div>
                    `;
                } catch (e) {
                    console.error("Load settings failed", e);
                }
            }
            
            async function saveSettings() {
                const provider = document.getElementById('editProvider').value;
                const base_url = document.getElementById('editBaseUrl').value;
                const model = document.getElementById('editModel').value;
                
                try {
                    const resp = await fetch('/api/config', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ provider, base_url, model })
                    });
                    
                    if (resp.ok) {
                        showToast('Settings Saved');
                    }
                } catch (e) {
                    console.error("Save settings failed", e);
                }
            }
            loadSettings();
        </script>
    "#;
    Html(layout(content, "settings"))
}
