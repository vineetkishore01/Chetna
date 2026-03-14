//! Web Dashboard - UI for memory visualization and management
//! Minimalist OLED black theme

use axum::{
    routing::get,
    Router,
    response::Html,
};
use std::sync::Arc;
use crate::{Brain, cache::SessionCache, config_file::UserConfig};

pub fn routes() -> Router<(Arc<Brain>, Arc<SessionCache>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(dashboard))
        .route("/memories", get(memories_page))
        .route("/skills", get(skills_page))
        .route("/sessions", get(sessions_page))
        .route("/settings", get(settings_page))
}

const STYLE: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", Roboto, sans-serif;
    background: #000000;
    color: #ffffff;
    min-height: 100vh;
    line-height: 1.5;
}
.header {
    background: #0a0a0a;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid #222;
    display: flex;
    align-items: center;
    gap: 2rem;
    position: sticky;
    top: 0;
    z-index: 100;
}
.header h1 { font-size: 1.25rem; font-weight: 600; }
.nav { display: flex; gap: 0.5rem; }
.nav a {
    color: #888;
    text-decoration: none;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.9rem;
    transition: color 0.15s;
}
.nav a:hover, .nav a.active { color: #fff; background: #1a1a1a; }
.container { padding: 1.5rem; max-width: 1200px; margin: 0 auto; }
.card {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 8px;
    padding: 1.25rem;
    margin-bottom: 1rem;
}
.card h2 { font-size: 1rem; font-weight: 600; margin-bottom: 1rem; color: #fff; }
.btn {
    background: #1a1a1a;
    color: #fff;
    border: 1px solid #333;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    transition: background 0.15s;
}
.btn:hover { background: #222; }
.btn-primary { background: #fff; color: #000; border-color: #fff; }
.btn-primary:hover { background: #e5e5e5; }
.btn-danger { background: #2a0a0a; border-color: #442222; color: #ff6b6b; }
.btn-danger:hover { background: #3a0a0a; }
input, select, textarea {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    color: #fff;
    font-size: 0.875rem;
    width: 100%;
}
input:focus, select:focus, textarea:focus {
    outline: none;
    border-color: #444;
}
label { display: block; color: #888; font-size: 0.8rem; margin-bottom: 0.35rem; }
.form-group { margin-bottom: 1rem; }
.form-row { display: flex; gap: 1rem; margin-bottom: 1rem; }
.form-row > * { flex: 1; }
.badge {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    background: #1a1a1a;
    color: #888;
}
.stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
}
.stat-card {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 8px;
    padding: 1rem;
    text-align: center;
}
.stat-card .value { font-size: 1.5rem; font-weight: 600; color: #fff; }
.stat-card .label { color: #666; font-size: 0.8rem; margin-top: 0.25rem; }
.toolbar {
    display: flex;
    gap: 0.75rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
    align-items: center;
}
.search-box { flex: 1; min-width: 200px; }
.item-list { display: flex; flex-direction: column; gap: 0.5rem; }
.item-card {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 8px;
    padding: 1rem;
}
.item-card:hover { border-color: #333; }
.item-meta { color: #666; font-size: 0.8rem; margin-bottom: 0.5rem; }
.item-content { color: #ddd; line-height: 1.6; }
.item-actions { display: flex; gap: 0.5rem; margin-top: 0.75rem; }
.empty-state { text-align: center; padding: 3rem; color: #444; }
.error-state { text-align: center; padding: 3rem; color: #ff6b6b; }
.loading { text-align: center; padding: 3rem; color: #666; }
.message { padding: 0.75rem; border-radius: 6px; margin-top: 1rem; font-size: 0.875rem; }
.message.success { background: #0a2a0a; color: #4ade80; }
.message.error { background: #2a0a0a; color: #ff6b6b; }
.pagination { display: flex; justify-content: center; gap: 0.5rem; margin-top: 1rem; }
.page-btn {
    padding: 0.4rem 0.75rem;
    background: #0a0a0a;
    border: 1px solid #222;
    color: #888;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
}
.page-btn:hover, .page-btn.active { background: #1a1a1a; color: #fff; border-color: #444; }
.page-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.modal-overlay {
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}
.modal {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
}
.modal h3 { margin-bottom: 1rem; }
.modal-actions { display: flex; gap: 0.5rem; margin-top: 1rem; justify-content: flex-end; }
code, pre {
    background: #0a0a0a;
    border: 1px solid #222;
    border-radius: 4px;
    font-family: "SF Mono", Monaco, "Cascadia Code", monospace;
    font-size: 0.8rem;
}
pre { padding: 0.75rem; overflow-x: auto; white-space: pre-wrap; }
code { padding: 0.1rem 0.3rem; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 0.75rem; text-align: left; border-bottom: 1px solid #222; }
th { color: #666; font-weight: 500; font-size: 0.8rem; text-transform: uppercase; }
tr:hover { background: #0a0a0a; }
.status-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-right: 0.5rem;
}
.status-dot.ok { background: #4ade80; }
.status-dot.error { background: #ff6b6b; }
.status-dot.warn { background: #fbbf24; }
.hidden { display: none !important; }
"#;

const HEADER_HTML: &str = r#"
<div class="header">
    <h1>Chetna</h1>
    <nav class="nav">
        <a href="/">Dashboard</a>
        <a href="/memories">Memories</a>
        <a href="/skills">Skills</a>
        <a href="/sessions">Sessions</a>
        <a href="/settings">Settings</a>
    </nav>
</div>
"#;

async fn dashboard() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Chetna</title>
<style>{STYLE}</style>
</head><body>
{HEADER_HTML}
<div class="container">
    <div class="stats-grid">
        <div class="stat-card"><div class="value" id="totalMemories">-</div><div class="label">Memories</div></div>
        <div class="stat-card"><div class="value" id="activeSessions">-</div><div class="label">Active Sessions</div></div>
        <div class="stat-card"><div class="value" id="totalSkills">-</div><div class="label">Skills</div></div>
        <div class="stat-card"><div class="value" id="totalProcedures">-</div><div class="label">Procedures</div></div>
    </div>

    <div class="card">
        <h2>🧠 Memory Operations</h2>
        <p style="color:#666;font-size:0.85rem;margin-bottom:1rem">
            Manual memory management. Auto-decay/flush can be enabled in <a href="/settings" style="color:#fff;text-decoration:underline">Settings</a>.
        </p>
        <div id="autoStatus" style="margin-bottom:1rem;padding:0.5rem;background:#1a1a1a;border-radius:6px;font-size:0.85rem;display:none"></div>
        <div class="toolbar">
            <button class="btn btn-primary" onclick="runConsolidate()">🔄 Run LLM Consolidation</button>
            <button class="btn" onclick="runDecay()">📉 Apply Decay Now</button>
            <button class="btn" onclick="runFlush()">🗑️ Flush Low Importance</button>
        </div>
        <div id="consolidateResults" class="message" style="display:none"></div>
    </div>

    <div class="card">
        <h2>🔍 Semantic Search</h2>
        <p style="color:#666;font-size:0.85rem;margin-bottom:1rem">
            Search memories by meaning using embeddings. Lower similarity threshold (0.3) finds more results.
        </p>
        <div class="toolbar">
            <input type="text" class="search-box" id="searchQuery" placeholder="Search memories (e.g., 'user preferences', 'coding habits')" style="flex:2">
            <input type="number" id="searchLimit" value="20" min="1" max="100" style="width:80px" title="Max results">
            <button class="btn btn-primary" onclick="runSearch()">Search</button>
        </div>
        <div id="searchResults"></div>
    </div>

    <div class="card">
        <h2>📦 Build Context for AI</h2>
        <p style="color:#666;font-size:0.85rem;margin-bottom:1rem">
            Build token-limited context from relevant memories for AI prompts.
        </p>
        <div class="toolbar">
            <input type="text" class="search-box" id="contextQuery" placeholder="Enter query (e.g., 'What are user preferences?')" style="flex:2">
            <select id="maxTokens" style="width:150px">
                <option value="500">500 tokens</option>
                <option value="1000" selected>1000 tokens</option>
                <option value="2000">2000 tokens</option>
                <option value="4000">4000 tokens</option>
            </select>
            <button class="btn" onclick="searchContext()">Build Context</button>
        </div>
        <div id="contextResults"></div>
    </div>

    <div class="card">
        <h2>🔗 Connection Status</h2>
        <div id="connectionStatus">Loading...</div>
    </div>
</div>

<script>
async function loadStats() {{
    try {{
        const r = await fetch('/api/stats');
        const d = await r.json();
        document.getElementById('totalMemories').textContent = d.total_memories || 0;
        document.getElementById('activeSessions').textContent = d.active_sessions || 0;
        document.getElementById('totalSkills').textContent = d.total_skills || 0;
        document.getElementById('totalProcedures').textContent = d.total_procedures || 0;
    }} catch(e) {{ console.error(e); }}
}}

async function loadAutoStatus() {{
    try {{
        const r = await fetch('/api/config/consolidation/status');
        const d = await r.json();

        const statusText = [];
        if (d.auto_decay_enabled) statusText.push('✅ Auto-decay enabled');
        else statusText.push('⚠️ Auto-decay disabled');

        if (d.auto_flush_enabled) statusText.push('✅ Auto-flush enabled');
        else statusText.push('⚠️ Auto-flush disabled');

        const statusEl = document.getElementById('autoStatus');
        if (statusEl) {{
            statusEl.innerHTML = statusText.join(' | ');
            statusEl.style.display = 'block';
        }}
    }} catch(e) {{ console.error('Failed to load auto status:', e); }}
}}

async function loadConnectionStatus() {{
    try {{
        const r = await fetch('/api/status/connections');
        const d = await r.json();
        const embed = d.embedding;
        const llm = d.llm;

        let html = '<div style="margin-bottom:0.5rem">';
        html += '<span class="status-dot ' + (embed.connected ? 'ok' : 'error') + '"></span>';
        html += 'Embedding: ' + (embed.connected ? 'Connected' : 'Disconnected');
        if (embed.model) html += ' (' + embed.model + ')';
        html += '</div>';

        html += '<div>';
        html += '<span class="status-dot ' + (llm.connected ? 'ok' : 'error') + '"></span>';
        html += 'LLM: ' + (llm.connected ? 'Connected' : 'Disconnected');
        if (llm.model) html += ' (' + llm.model + ')';
        html += '</div>';

        document.getElementById('connectionStatus').innerHTML = html;
    }} catch(e) {{
        document.getElementById('connectionStatus').innerHTML = '<span class="status-dot error"></span>Error loading status';
    }}
}}

async function searchContext() {{
    const query = document.getElementById('contextQuery').value.trim();
    const maxTokens = parseInt(document.getElementById('maxTokens').value);
    const resultsDiv = document.getElementById('contextResults');

    if (!query) {{
        resultsDiv.innerHTML = '<div class="message error">Please enter a query</div>';
        return;
    }}

    resultsDiv.innerHTML = '<div class="loading">Building context from memories...</div>';

    try {{
        const r = await fetch('/api/memory/context', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ query: query, max_tokens: maxTokens }})
        }});

        if (!r.ok) throw new Error('HTTP ' + r.status);

        const data = await r.json();

        if (!data.memories || data.memories.length === 0) {{
            resultsDiv.innerHTML = '<div class="empty-state">No relevant memories found. Try adding some memories first!</div>';
            return;
        }}

        let html = '<div style="margin-bottom:1rem;padding:0.75rem;background:#0a2a0a;border-radius:6px;border:1px solid #1a3a1a">';
        html += '<div style="color:#4ade80;font-size:0.85rem;margin-bottom:0.5rem">✓ Found ' + data.memories.length + ' relevant memories</div>';
        html += '<div style="color:#666;font-size:0.8rem">Total tokens: ' + data.total_tokens + ' | Context length: ' + data.context.length + ' chars</div>';
        html += '</div>';

        html += '<div style="display:flex;flex-direction:column;gap:0.75rem">';

        data.memories.forEach((m, i) => {{
            const importanceColor = m.importance >= 0.7 ? '#4ade80' : (m.importance >= 0.4 ? '#fbbf24' : '#666');
            html += '<div class="item-card" style="border-left:3px solid ' + importanceColor + '">';
            html += '<div class="item-meta">';
            html += '<span class="badge" style="background:' + importanceColor + ';color:#000">' + m.importance.toFixed(2) + '</span>';
            html += '<span class="badge">' + m.memory_type + '</span>';
            html += '<span class="badge">' + m.category + '</span>';
            if (m.embedding_model) html += '<span class="badge">📎 ' + m.embedding_model + '</span>';
            html += '</div>';
            html += '<div class="item-content" style="margin-top:0.5rem">' + escapeHtml(m.content) + '</div>';
            html += '</div>';
        }});

        html += '</div>';

        resultsDiv.innerHTML = html;

    }} catch(e) {{
        resultsDiv.innerHTML = '<div class="message error">Error: ' + e.message + '</div>';
    }}
}}

async function runSearch() {{
    const query = document.getElementById('searchQuery').value.trim();
    const limit = parseInt(document.getElementById('searchLimit').value) || 20;
    const resultsDiv = document.getElementById('searchResults');

    if (!query) {{
        resultsDiv.innerHTML = '<div class="message error">Please enter a search query</div>';
        return;
    }}

    resultsDiv.innerHTML = '<div class="loading">Searching memories...</div>';

    try {{
        const r = await fetch('/api/memory/search?query=' + encodeURIComponent(query) + '&limit=' + limit);
        if (!r.ok) throw new Error('HTTP ' + r.status);

        const memories = await r.json();

        if (memories.length === 0) {{
            resultsDiv.innerHTML = '<div class="empty-state">No memories found matching "' + escapeHtml(query) + '"</div>';
            return;
        }}

        let html = '<div style="margin-bottom:1rem;padding:0.75rem;background:#0a2a0a;border-radius:6px;border:1px solid #1a3a1a">';
        html += '<div style="color:#4ade80;font-size:0.85rem">✓ Found ' + memories.length + ' memories</div>';
        html += '</div>';

        html += '<div style="display:flex;flex-direction:column;gap:0.75rem">';

        memories.forEach(m => {{
            const importanceColor = m.importance >= 0.7 ? '#4ade80' : (m.importance >= 0.4 ? '#fbbf24' : '#666');
            html += '<div class="item-card" style="border-left:3px solid ' + importanceColor + '">';
            html += '<div class="item-meta">';
            html += '<span class="badge" style="background:' + importanceColor + ';color:#000">' + m.importance.toFixed(2) + '</span>';
            html += '<span class="badge">' + m.memory_type + '</span>';
            html += '<span class="badge">' + m.category + '</span>';
            html += '<span style="float:right">' + new Date(m.created_at).toLocaleDateString() + '</span>';
            html += '</div>';
            html += '<div class="item-content" style="margin-top:0.5rem">' + escapeHtml(m.content) + '</div>';
            html += '</div>';
        }});

        html += '</div>';
        resultsDiv.innerHTML = html;

    }} catch(e) {{
        resultsDiv.innerHTML = '<div class="message error">Error: ' + e.message + '</div>';
    }}
}}

function escapeHtml(t) {{
    const d = document.createElement('div');
    d.textContent = t;
    return d.innerHTML;
}}

async function runConsolidate() {{
    const resultsDiv = document.getElementById('consolidateResults');
    resultsDiv.style.display = 'block';
    resultsDiv.className = 'message';
    resultsDiv.textContent = '🔄 Running LLM consolidation (this may take a while)...';

    try {{
        const r = await fetch('/api/memory/consolidate', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ limit: 50 }})
        }});

        const data = await r.json();

        if (data.success) {{
            resultsDiv.className = 'message success';
            resultsDiv.innerHTML = '✅ ' + data.message;
        }} else {{
            resultsDiv.className = 'message error';
            resultsDiv.textContent = '❌ ' + data.message;
        }}
    }} catch(e) {{
        resultsDiv.className = 'message error';
        resultsDiv.textContent = '❌ Error: ' + e.message;
    }}
}}

async function runDecay() {{
    const resultsDiv = document.getElementById('consolidateResults');
    resultsDiv.style.display = 'block';
    resultsDiv.className = 'message';
    resultsDiv.textContent = '📉 Applying Ebbinghaus decay formula...';

    try {{
        const r = await fetch('/api/memory/decay', {{ method: 'POST' }});
        const data = await r.json();

        if (data.success) {{
            resultsDiv.className = 'message success';
            resultsDiv.innerHTML = '✅ ' + data.message;
        }} else {{
            resultsDiv.className = 'message error';
            resultsDiv.textContent = '❌ ' + data.message;
        }}
    }} catch(e) {{
        resultsDiv.className = 'message error';
        resultsDiv.textContent = '❌ Error: ' + e.message;
    }}
}}

async function runFlush() {{
    const resultsDiv = document.getElementById('consolidateResults');
    resultsDiv.style.display = 'block';
    resultsDiv.className = 'message';
    resultsDiv.textContent = '🗑️ Flushing low-importance memories...';

    try {{
        const r = await fetch('/api/memory/flush', {{ method: 'POST' }});
        const data = await r.json();

        if (data.success) {{
            resultsDiv.className = 'message success';
            resultsDiv.innerHTML = '✅ ' + data.message;
        }} else {{
            resultsDiv.className = 'message error';
            resultsDiv.textContent = '❌ ' + data.message;
        }}
    }} catch(e) {{
        resultsDiv.className = 'message error';
        resultsDiv.textContent = '❌ Error: ' + e.message;
    }}
}}

document.getElementById('searchQuery').addEventListener('keypress', e => {{
    if (e.key === 'Enter') runSearch();
}});

document.getElementById('contextQuery').addEventListener('keypress', e => {{
    if (e.key === 'Enter') searchContext();
}});

loadStats();
loadAutoStatus();
loadConnectionStatus();
</script>
</body></html>"#))
}

async fn memories_page() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Memories - Chetna</title>
<style>{STYLE}</style>
</head><body>
{HEADER_HTML}
<div class="container">
    <div class="toolbar">
        <input type="text" class="search-box" id="searchBox" placeholder="Search memories... (press / to focus)">
        <button class="btn" onclick="showCreateModal()">+ New</button>
        <button class="btn" onclick="loadMemories()">Refresh</button>
    </div>
    
    <div id="memories" class="item-list"><div class="loading">Loading...</div></div>
    <div class="pagination" id="pagination"></div>
</div>

<div id="createModal" class="modal-overlay hidden">
    <div class="modal">
        <h3>New Memory</h3>
        <div class="form-group">
            <label>Content</label>
            <textarea id="newContent" rows="4" placeholder="Enter memory content..."></textarea>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label>Type</label>
                <select id="newType">
                    <option value="fact">Fact</option>
                    <option value="preference">Preference</option>
                    <option value="rule">Rule</option>
                    <option value="experience">Experience</option>
                </select>
            </div>
            <div class="form-group">
                <label>Importance</label>
                <input type="number" id="newImportance" min="0" max="1" step="0.1" value="0.5">
            </div>
        </div>
        <div class="modal-actions">
            <button class="btn" onclick="hideCreateModal()">Cancel</button>
            <button class="btn btn-primary" onclick="createMemory()">Create</button>
        </div>
    </div>
</div>

<script>
let allMemories = [], filteredMemories = [];
const PAGE_SIZE = 20;
let currentPage = 1;

function showCreateModal() {{ document.getElementById('createModal').classList.remove('hidden'); }}
function hideCreateModal() {{ document.getElementById('createModal').classList.add('hidden'); }}

function renderPage(page) {{
    const start = (page - 1) * PAGE_SIZE;
    const end = start + PAGE_SIZE;
    const items = filteredMemories.slice(start, end);
    const container = document.getElementById('memories');
    
    if (items.length === 0) {{
        container.innerHTML = '<div class="empty-state">' + (filteredMemories.length === 0 ? 'No memories yet' : 'No matches') + '</div>';
        document.getElementById('pagination').innerHTML = '';
        return;
    }}
    
    container.innerHTML = items.map(m => `
        <div class="item-card">
            <div class="item-meta">
                <span class="badge">${{m.importance.toFixed(2)}}</span>
                <span class="badge">${{m.memory_type}}</span>
                <span class="badge">${{m.category}}</span>
                <span style="float:right">${{new Date(m.created_at).toLocaleDateString()}}</span>
            </div>
            <div class="item-content">${{escapeHtml(m.content)}}</div>
            <div class="item-actions">
                <button class="btn" onclick="togglePin('${{m.id}}', ${{!m.is_pinned}})">${{m.is_pinned ? 'Unpin' : 'Pin'}}</button>
                <button class="btn btn-danger" onclick="deleteMemory('${{m.id}}')">Delete</button>
            </div>
        </div>
    `).join('');
    
    renderPagination(page);
}}

function renderPagination(current) {{
    const total = Math.ceil(filteredMemories.length / PAGE_SIZE);
    if (total <= 1) {{ document.getElementById('pagination').innerHTML = ''; return; }}
    
    let html = '';
    html += `<button class="page-btn" ${{current===1?'disabled':''}} onclick="goToPage(${{current-1}})">Prev</button>`;
    for (let i = Math.max(1, current-2); i <= Math.min(total, current+2); i++) {{
        html += `<button class="page-btn${{i===current?' active':''}}" onclick="goToPage(${{i}})">${{i}}</button>`;
    }}
    html += `<button class="page-btn" ${{current===total?'disabled':''}} onclick="goToPage(${{current+1}})">Next</button>`;
    document.getElementById('pagination').innerHTML = html;
}}

function goToPage(p) {{ currentPage = p; renderPage(p); }}
function escapeHtml(t) {{ const d = document.createElement('div'); d.textContent = t; return d.innerHTML; }}

async function loadMemories() {{
    document.getElementById('memories').innerHTML = '<div class="loading">Loading...</div>';
    try {{
        const r = await fetch('/api/memory?limit=1000');
        if (!r.ok) throw new Error('HTTP ' + r.status);
        allMemories = await r.json();
        filteredMemories = [...allMemories];
        currentPage = 1;
        renderPage(1);
    }} catch(e) {{
        document.getElementById('memories').innerHTML = '<div class="error-state">Error: ' + e.message + '</div>';
    }}
}}

function searchMemories(q) {{
    q = q.toLowerCase().trim();
    filteredMemories = q ? allMemories.filter(m => 
        m.content.toLowerCase().includes(q) || 
        m.memory_type.toLowerCase().includes(q) ||
        m.category.toLowerCase().includes(q)
    ) : [...allMemories];
    currentPage = 1;
    renderPage(1);
}}

async function createMemory() {{
    const content = document.getElementById('newContent').value.trim();
    if (!content) {{ alert('Enter content'); return; }}
    
    try {{
        const r = await fetch('/api/memory', {{
            method: 'POST',
            headers: {{'Content-Type': 'application/json'}},
            body: JSON.stringify({{
                content: content,
                memory_type: document.getElementById('newType').value,
                importance: parseFloat(document.getElementById('newImportance').value)
            }})
        }});
        if (!r.ok) throw new Error('Failed');
        hideCreateModal();
        document.getElementById('newContent').value = '';
        loadMemories();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

async function deleteMemory(id) {{
    if (!confirm('Delete this memory?')) return;
    try {{
        const r = await fetch('/api/memory/' + id, {{ method: 'DELETE' }});
        if (!r.ok) throw new Error('Failed');
        loadMemories();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

async function togglePin(id, pinned) {{
    try {{
        const r = await fetch('/api/memory/pin/' + id, {{ method: pinned ? 'POST' : 'DELETE' }});
        if (!r.ok) throw new Error('Failed');
        loadMemories();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

document.getElementById('searchBox').addEventListener('input', e => {{
    clearTimeout(window._searchTimeout);
    window._searchTimeout = setTimeout(() => searchMemories(e.target.value), 300);
}});

document.addEventListener('keydown', e => {{
    if (e.key === '/' && document.activeElement.tagName !== 'INPUT' && document.activeElement.tagName !== 'TEXTAREA') {{
        e.preventDefault();
        document.getElementById('searchBox').focus();
    }}
}});

loadMemories();
</script>
</body></html>"#))
}

async fn skills_page() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Skills - Chetna</title>
<style>{STYLE}</style>
</head><body>
{HEADER_HTML}
<div class="container">
    <div class="toolbar">
        <input type="text" class="search-box" id="searchBox" placeholder="Search skills...">
        <button class="btn" onclick="alert('Use API to create skills')">+ New</button>
        <button class="btn" onclick="loadSkills()">Refresh</button>
    </div>
    <div id="skills" class="item-list"><div class="loading">Loading...</div></div>
</div>
<script>
async function loadSkills() {{
    document.getElementById('skills').innerHTML = '<div class="loading">Loading...</div>';
    try {{
        const r = await fetch('/api/skill');
        if (!r.ok) throw new Error('HTTP ' + r.status);
        const skills = await r.json();
        
        if (skills.length === 0) {{
            document.getElementById('skills').innerHTML = '<div class="empty-state">No skills yet</div>';
            return;
        }}
        
        document.getElementById('skills').innerHTML = skills.map(s => `
            <div class="item-card">
                <div class="item-meta"><span class="badge">${{s.language}}</span></div>
                <div style="font-weight:600;margin-bottom:0.5rem">${{escapeHtml(s.name)}}</div>
                <div class="item-content">${{escapeHtml(s.description || 'No description')}}</div>
                <pre>${{escapeHtml(s.code.substring(0, 500))}}</pre>
                <div class="item-actions">
                    <button class="btn btn-danger" onclick="deleteSkill('${{s.id}}')">Delete</button>
                </div>
            </div>
        `).join('');
    }} catch(e) {{
        document.getElementById('skills').innerHTML = '<div class="error-state">Error: ' + e.message + '</div>';
    }}
}}

function escapeHtml(t) {{ const d = document.createElement('div'); d.textContent = t; return d.innerHTML; }}

async function deleteSkill(id) {{
    if (!confirm('Delete this skill?')) return;
    try {{
        const r = await fetch('/api/skill/' + id, {{ method: 'DELETE' }});
        if (!r.ok) throw new Error('Failed');
        loadSkills();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

document.getElementById('searchBox').addEventListener('input', e => {{
    const q = e.target.value.toLowerCase();
    document.querySelectorAll('.item-card').forEach(card => {{
        card.style.display = card.textContent.toLowerCase().includes(q) ? '' : 'none';
    }});
}});

loadSkills();
</script>
</body></html>"#))
}

async fn sessions_page() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Sessions - Chetna</title>
<style>{STYLE}</style>
</head><body>
{HEADER_HTML}
<div class="container">
    <div class="toolbar">
        <input type="text" class="search-box" id="searchBox" placeholder="Search sessions...">
        <button class="btn" onclick="createSession()">+ New</button>
        <button class="btn" onclick="loadSessions()">Refresh</button>
    </div>
    <div id="sessions" class="item-list"><div class="loading">Loading...</div></div>
</div>
<script>
async function loadSessions() {{
    document.getElementById('sessions').innerHTML = '<div class="loading">Loading...</div>';
    try {{
        const r = await fetch('/api/session');
        if (!r.ok) throw new Error('HTTP ' + r.status);
        const sessions = await r.json();
        
        if (sessions.length === 0) {{
            document.getElementById('sessions').innerHTML = '<div class="empty-state">No sessions yet</div>';
            return;
        }}
        
        document.getElementById('sessions').innerHTML = sessions.map(s => {{
            const active = !s.ended_at;
            return `
            <div class="item-card">
                <div class="item-meta">
                    <span class="badge" style="background:${{active ? '#0a2a0a' : '#1a1a1a'}}">${{active ? 'Active' : 'Ended'}}</span>
                    <span style="float:right">${{new Date(s.started_at).toLocaleString()}}</span>
                </div>
                <div style="font-weight:600">${{escapeHtml(s.name)}}</div>
                <div class="item-actions">
                    ${{active ? `<button class="btn" onclick="endSession('${{s.id}}')">End</button>` : ''}}
                    <button class="btn btn-danger" onclick="deleteSession('${{s.id}}')">Delete</button>
                </div>
            </div>
            `;
        }}).join('');
    }} catch(e) {{
        document.getElementById('sessions').innerHTML = '<div class="error-state">Error: ' + e.message + '</div>';
    }}
}}

function escapeHtml(t) {{ const d = document.createElement('div'); d.textContent = t; return d.innerHTML; }}

async function createSession() {{
    const name = prompt('Session name:');
    if (!name) return;
    try {{
        const r = await fetch('/api/session', {{
            method: 'POST',
            headers: {{'Content-Type': 'application/json'}},
            body: JSON.stringify({{ name }})
        }});
        if (!r.ok) throw new Error('Failed');
        loadSessions();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

async function endSession(id) {{
    try {{
        const r = await fetch('/api/session/' + id + '/end', {{ method: 'POST' }});
        if (!r.ok) throw new Error('Failed');
        loadSessions();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

async function deleteSession(id) {{
    if (!confirm('Delete this session?')) return;
    try {{
        const r = await fetch('/api/session/' + id, {{ method: 'DELETE' }});
        if (!r.ok) throw new Error('Failed');
        loadSessions();
    }} catch(e) {{ alert('Error: ' + e.message); }}
}}

loadSessions();
</script>
</body></html>"#))
}

async fn settings_page() -> Html<String> {
    Html(format!(r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Settings - Chetna</title>
<style>{STYLE}</style>
</head><body>
{HEADER_HTML}
<div class="container">
    <div class="card">
        <h2>Embedding Settings</h2>
        <div class="form-row">
            <div class="form-group">
                <label>Ollama URL</label>
                <input type="text" id="embedUrl" placeholder="http://localhost:11434">
            </div>
            <div class="form-group">
                <label>Model</label>
                <select id="embedModel">
                    <option value="">Click refresh to load models...</option>
                </select>
            </div>
        </div>
        <button class="btn" onclick="testEmbed()">🔄 Refresh Models & Test</button>
        <button class="btn btn-primary" onclick="saveEmbed()">💾 Save</button>
        <div id="embedMsg" class="message" style="display:none"></div>
    </div>

    <div class="card">
        <h2>LLM Settings</h2>
        <div class="form-row">
            <div class="form-group">
                <label>Ollama URL</label>
                <input type="text" id="llmUrl" placeholder="http://localhost:11434">
            </div>
            <div class="form-group">
                <label>Model</label>
                <select id="llmModel">
                    <option value="">Click refresh to load models...</option>
                </select>
            </div>
        </div>
        <button class="btn" onclick="testLlm()">🔄 Refresh Models & Test</button>
        <button class="btn btn-primary" onclick="saveLlm()">💾 Save</button>
        <div id="llmMsg" class="message" style="display:none"></div>
    </div>

    <div class="card">
        <h2>Consolidation Settings</h2>
        <div class="form-row">
            <div class="form-group">
                <label>Interval (hours)</label>
                <input type="number" id="consolidInterval" min="1" max="168" value="6">
            </div>
            <div class="form-group">
                <label>Min Importance Threshold</label>
                <input type="number" id="minImportance" min="0" max="1" step="0.05" value="0.1">
            </div>
        </div>
        <div class="form-row">
            <div class="form-group">
                <label>
                    <input type="checkbox" id="autoDecay" checked> Auto Decay (Ebbinghaus)
                </label>
            </div>
            <div class="form-group">
                <label>
                    <input type="checkbox" id="autoFlush" checked> Auto Flush Low Importance
                </label>
            </div>
        </div>
        <button class="btn btn-primary" onclick="saveConsolidation()">Save Settings</button>
        <div id="consolidMsg" class="message" style="display:none"></div>
    </div>

    <div class="card">
        <h2>Connection Status</h2>
        <div id="connectionStatus">Loading...</div>
    </div>
</div>

<script>
let embedModels = [];
let llmModels = [];

function showMsg(id, msg, isError = false) {{
    const el = document.getElementById(id);
    el.style.display = 'block';
    el.className = 'message ' + (isError ? 'error' : 'success');
    el.textContent = msg;
}}

async function loadSettings() {{
    try {{
        const r = await fetch('/api/config/user');
        const cfg = await r.json();
        
        if (cfg.embedding_base_url) document.getElementById('embedUrl').value = cfg.embedding_base_url;
        if (cfg.embedding_model) document.getElementById('embedModel').innerHTML = '<option value="">' + cfg.embedding_model + ' (saved)</option>';
        if (cfg.llm_base_url) document.getElementById('llmUrl').value = cfg.llm_base_url;
        if (cfg.llm_model) document.getElementById('llmModel').innerHTML = '<option value="">' + cfg.llm_model + ' (saved)</option>';
        if (cfg.consolidation_interval_hours) document.getElementById('consolidInterval').value = cfg.consolidation_interval_hours;
        if (cfg.min_importance_threshold) document.getElementById('minImportance').value = cfg.min_importance_threshold;
        document.getElementById('autoDecay').checked = cfg.auto_decay_enabled !== false;
        document.getElementById('autoFlush').checked = cfg.auto_flush_enabled !== false;
        
        loadConnectionStatus();
    }} catch(e) {{ console.error(e); }}
}}

async function loadConnectionStatus() {{
    try {{
        const r = await fetch('/api/status/connections');
        const d = await r.json();
        
        let html = '';
        
        // Embedding status
        html += '<div style="margin-bottom:1rem;padding:0.75rem;background:#0a0a0a;border-radius:6px;border:1px solid #222">';
        if (d.embedding.configured) {{
            const embedColor = d.embedding.connected ? '#4ade80' : '#ff6b6b';
            html += '<div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem">';
            html += '<span class="status-dot" style="background:' + embedColor + '"></span>';
            html += '<span style="color:#fff;font-weight:600">Embedding: ' + (d.embedding.connected ? '✅ Connected' : '❌ Disconnected') + '</span>';
            html += '</div>';
            html += '<div style="color:#888;font-size:0.8rem;margin-left:1rem">';
            if (d.embedding.model) html += '<div>Model: ' + d.embedding.model + '</div>';
            if (d.embedding.base_url) html += '<div>URL: ' + d.embedding.base_url + '</div>';
            html += '</div>';
        }} else {{
            html += '<span style="color:#666">Not configured - Click "Refresh Models" to set up</span>';
        }}
        html += '</div>';
        
        // LLM status
        html += '<div style="padding:0.75rem;background:#0a0a0a;border-radius:6px;border:1px solid #222">';
        if (d.llm.configured) {{
            const llmColor = d.llm.connected ? '#4ade80' : '#ff6b6b';
            html += '<div style="display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem">';
            html += '<span class="status-dot" style="background:' + llmColor + '"></span>';
            html += '<span style="color:#fff;font-weight:600">LLM: ' + (d.llm.connected ? '✅ Connected' : '❌ Disconnected') + '</span>';
            html += '</div>';
            html += '<div style="color:#888;font-size:0.8rem;margin-left:1rem">';
            if (d.llm.model) html += '<div>Model: ' + d.llm.model + '</div>';
            if (d.llm.base_url) html += '<div>URL: ' + d.llm.base_url + '</div>';
            html += '</div>';
        }} else {{
            html += '<span style="color:#666">Not configured - Click "Refresh Models" to set up</span>';
        }}
        html += '</div>';
        
        document.getElementById('connectionStatus').innerHTML = html;
    }} catch(e) {{
        document.getElementById('connectionStatus').innerHTML = '<span style="color:#ff6b6b">Error loading status</span>';
    }}
}}

async function testEmbed() {{
    const url = document.getElementById('embedUrl').value || 'http://localhost:11434';
    const msg = document.getElementById('embedMsg');
    msg.style.display = 'block';
    msg.className = 'message';
    msg.textContent = '🔄 Connecting to Ollama...';
    
    try {{
        const r = await fetch('/api/config/embedding/test', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ base_url: url }})
        }});
        const d = await r.json();
        
        // Update the models dropdown
        const select = document.getElementById('embedModel');
        select.innerHTML = '';
        
        if (d.available_models && d.available_models.length > 0) {{
            embedModels = d.available_models;
            d.available_models.forEach(m => {{
                const opt = document.createElement('option');
                opt.value = m;
                opt.textContent = m;
                select.appendChild(opt);
            }});
        }} else {{
            select.innerHTML = '<option value="">No models found</option>';
        }}
        
        if (d.success && d.connected) {{
            msg.className = 'message success';
            if (d.model_installed && d.embedding_works) {{
                msg.innerHTML = '✅ <strong>Connected!</strong> Model: ' + d.model + ' - Embedding works!';
                // Auto-select the working model
                if (d.model) select.value = d.model;
            }} else if (d.model_installed) {{
                msg.innerHTML = '⚠️ <strong>Connected but test failed</strong> - Select model and try again';
            }} else {{
                msg.innerHTML = '✅ <strong>Connected to Ollama!</strong> Select a model from the dropdown.';
            }}
        }} else {{
            msg.className = 'message error';
            msg.textContent = '❌ ' + d.message;
        }}
    }} catch(e) {{
        msg.className = 'message error';
        msg.textContent = '❌ Error: ' + e.message;
    }}
}}

async function testLlm() {{
    const url = document.getElementById('llmUrl').value || 'http://localhost:11434';
    const msg = document.getElementById('llmMsg');
    msg.style.display = 'block';
    msg.className = 'message';
    msg.textContent = '🔄 Connecting to Ollama...';
    
    try {{
        const r = await fetch('/api/config/llm/test', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{ base_url: url, model: '' }})
        }});
        const d = await r.json();
        
        // Update the models dropdown
        const select = document.getElementById('llmModel');
        select.innerHTML = '';
        
        if (d.available_models && d.available_models.length > 0) {{
            llmModels = d.available_models;
            d.available_models.forEach(m => {{
                const opt = document.createElement('option');
                opt.value = m;
                opt.textContent = m;
                select.appendChild(opt);
            }});
        }} else {{
            select.innerHTML = '<option value="">No models found</option>';
        }}
        
        if (d.success && d.connected) {{
            msg.className = 'message success';
            if (d.model_installed && d.llm_works) {{
                msg.innerHTML = '✅ <strong>Connected!</strong> Model: ' + d.model + ' - Chat works!';
                // Auto-select the working model
                if (d.model) select.value = d.model;
            }} else if (d.model_installed) {{
                msg.innerHTML = '⚠️ <strong>Connected but test failed</strong> - Select model and try again';
            }} else {{
                msg.innerHTML = '✅ <strong>Connected to Ollama!</strong> Select a model from the dropdown.';
            }}
        }} else {{
            msg.className = 'message error';
            msg.textContent = '❌ ' + d.message;
        }}
    }} catch(e) {{
        msg.className = 'message error';
        msg.textContent = '❌ Error: ' + e.message;
    }}
}}

async function saveEmbed() {{
    const url = document.getElementById('embedUrl').value;
    const model = document.getElementById('embedModel').value;
    
    if (!model) {{
        showMsg('embedMsg', 'Please select a model first (click "Refresh Models")', true);
        return;
    }}
    
    try {{
        const r = await fetch('/api/config/user', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                embedding_base_url: url,
                embedding_model: model
            }})
        }});
        const d = await r.json();
        
        if (d.success) {{
            showMsg('embedMsg', '✅ Saved! Restart Chetna to apply changes.');
            setTimeout(loadConnectionStatus, 500);
        }} else {{
            showMsg('embedMsg', '❌ ' + (d.message || 'Failed to save'), true);
        }}
    }} catch(e) {{
        showMsg('embedMsg', '❌ Error: ' + e.message, true);
    }}
}}

async function saveLlm() {{
    const url = document.getElementById('llmUrl').value;
    const model = document.getElementById('llmModel').value;
    
    if (!model) {{
        showMsg('llmMsg', 'Please select a model first (click "Refresh Models")', true);
        return;
    }}
    
    try {{
        const r = await fetch('/api/config/user', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                llm_base_url: url,
                llm_model: model
            }})
        }});
        const d = await r.json();
        
        if (d.success) {{
            showMsg('llmMsg', '✅ Saved! Restart Chetna to apply changes.');
            setTimeout(loadConnectionStatus, 500);
        }} else {{
            showMsg('llmMsg', '❌ ' + (d.message || 'Failed to save'), true);
        }}
    }} catch(e) {{
        showMsg('llmMsg', '❌ Error: ' + e.message, true);
    }}
}}

async function saveConsolidation() {{
    try {{
        const r = await fetch('/api/config/user', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                consolidation_interval_hours: parseInt(document.getElementById('consolidInterval').value),
                min_importance_threshold: parseFloat(document.getElementById('minImportance').value),
                auto_decay_enabled: document.getElementById('autoDecay').checked,
                auto_flush_enabled: document.getElementById('autoFlush').checked
            }})
        }});
        const d = await r.json();
        
        const msg = document.getElementById('consolidMsg');
        msg.style.display = 'block';
        msg.className = 'message ' + (d.success ? 'success' : 'error');
        msg.textContent = d.success ? '✅ ' + d.message : '❌ ' + d.message;
    }} catch(e) {{
        alert('Error: ' + e.message);
    }}
}}

// Auto-load settings on page load
loadSettings();
</script>
</body></html>"#))
}
