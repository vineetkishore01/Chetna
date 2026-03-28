//! History web UI pages for tracking memory operations

use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;
use crate::{Brain, config_file::UserConfig};

pub fn routes() -> Router<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)> {
    Router::new()
        .route("/", get(history_page))
        .route("/analytics", get(analytics_page))
        .route("/:id", get(event_details_page))
}

/// Main history timeline page
async fn history_page(
    State((_brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> Html<String> {
    let content = r#"
        <h2>History</h2>
        <p style="color: #888; margin-bottom: 2rem;">Timeline of all memory operations and queries.</p>
        
        <div style="margin-bottom: 2rem; display: flex; gap: 1rem; flex-wrap: wrap;">
            <select id="eventTypeFilter" style="background: #111; border: 1px solid #222; color: #fff; padding: 0.5rem; border-radius: 6px;">
                <option value="">All Events</option>
                <option value="memory_created">Memory Created</option>
                <option value="query_searched">Query Searched</option>
                <option value="context_built">Context Built</option>
            </select>
            <input type="text" id="searchFilter" placeholder="Search events..." style="background: #111; border: 1px solid #222; color: #fff; padding: 0.5rem; border-radius: 6px; flex: 1; min-width: 200px;">
            <button onclick="loadHistory()" style="background: #fff; color: #000; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer;">Filter</button>
        </div>

        <div id="historyList" style="display: flex; flex-direction: column; gap: 1rem;"></div>
        
        <div id="pagination" style="margin-top: 2rem; display: flex; justify-content: center; gap: 1rem; align-items: center;">
            <button onclick="loadPreviousPage()" id="prevBtn" disabled style="background: #111; color: #888; border: 1px solid #222; padding: 0.5rem 1rem; border-radius: 6px; cursor: not-allowed;">Previous</button>
            <span id="pageInfo" style="color: #888;">Page 1</span>
            <button onclick="loadNextPage()" id="nextBtn" style="background: #111; color: #fff; border: 1px solid #222; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer;">Next</button>
        </div>

        <script>
            let currentPage = 0;
            const pageSize = 50;

            async function loadHistory() {
                const eventType = document.getElementById('eventTypeFilter').value;
                const search = document.getElementById('searchFilter').value;
                
                let url = `/api/history?limit=${pageSize}&offset=${currentPage * pageSize}`;
                if (eventType) url += `&event_type=${eventType}`;
                
                try {
                    const resp = await fetch(url);
                    const data = await resp.json();
                    
                    const container = document.getElementById('historyList');
                    if (data.events.length === 0) {
                        container.innerHTML = '<div style="color: #444; text-align: center; padding: 2rem;">No events found.</div>';
                        return;
                    }
                    
                    container.innerHTML = data.events.map(event => {
                        const timestamp = new Date(event.timestamp).toLocaleString();
                        const eventType = event.event_type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase());
                        
                        let details = '';
                        if (event.event_type === 'memory_created') {
                            details = `
                                <div style="margin-top: 0.5rem; padding: 0.75rem; background: #111; border-radius: 6px;">
                                    <div style="font-size: 0.85rem; color: #aaa; margin-bottom: 0.5rem;">Memory Details</div>
                                    <div style="font-size: 0.9rem; color: #eee; line-height: 1.4;">${event.metadata.content || 'N/A'}</div>
                                    <div style="margin-top: 0.5rem; font-size: 0.75rem; color: #666;">
                                        Category: ${event.metadata.category || 'N/A'} | 
                                        Importance: ${event.metadata.importance || 'N/A'}
                                    </div>
                                </div>
                            `;
                        } else if (event.event_type === 'query_searched') {
                            details = `
                                <div style="margin-top: 0.5rem; padding: 0.75rem; background: #111; border-radius: 6px;">
                                    <div style="font-size: 0.85rem; color: #aaa; margin-bottom: 0.5rem;">Query Details</div>
                                    <div style="font-size: 0.9rem; color: #eee; line-height: 1.4;">"${event.metadata.query || 'N/A'}"</div>
                                    <div style="margin-top: 0.5rem; font-size: 0.75rem; color: #666;">
                                        Results: ${event.metadata.results_count || 0} | 
                                        Duration: ${event.metadata.duration_ms || 0}ms
                                    </div>
                                </div>
                            `;
                        }
                        
                        return `
                            <div style="background: #0a0a0a; padding: 1.25rem; border-radius: 10px; border: 1px solid #222; cursor: pointer;" onclick="showEventDetails('${event.id}')">
                                <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem;">
                                    <span style="font-size: 0.65rem; background: #1a1a1a; color: #888; padding: 0.2rem 0.5rem; border-radius: 4px; text-transform: uppercase; letter-spacing: 0.05em;">${eventType}</span>
                                    <span style="font-size: 0.65rem; color: #444;">${timestamp}</span>
                                </div>
                                ${details}
                            </div>
                        `;
                    }).join('');
                    
                    // Update pagination
                    document.getElementById('pageInfo').innerText = `Page ${currentPage + 1}`;
                    document.getElementById('prevBtn').disabled = currentPage === 0;
                    document.getElementById('nextBtn').disabled = data.events.length < pageSize;
                    
                } catch (e) {
                    console.error("Load history failed", e);
                }
            }
            
            function loadPreviousPage() {
                if (currentPage > 0) {
                    currentPage--;
                    loadHistory();
                }
            }
            
            function loadNextPage() {
                currentPage++;
                loadHistory();
            }
            
            function showEventDetails(eventId) {
                window.location.href = `/history/${eventId}`;
            }
            
            loadHistory();
        </script>
    "#;
    
    Html(content.to_string())
}

/// Analytics dashboard page
async fn analytics_page(
    State((_brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
) -> Html<String> {
    let content = r#"
        <h2>Analytics</h2>
        <p style="color: #888; margin-bottom: 2rem;">Insights into memory operations and query patterns.</p>
        
        <div style="margin-bottom: 2rem;">
            <label style="color: #888; margin-right: 1rem;">Time Range:</label>
            <select id="timeRange" onchange="loadAnalytics()" style="background: #111; border: 1px solid #222; color: #fff; padding: 0.5rem; border-radius: 6px;">
                <option value="7">Last 7 days</option>
                <option value="30" selected>Last 30 days</option>
                <option value="90">Last 90 days</option>
            </select>
        </div>

        <div class="grid" style="margin-bottom: 2rem;">
            <div class="stat-box">
                <div class="label">Total Events</div>
                <div class="value" id="totalEvents">0</div>
            </div>
            <div class="stat-box">
                <div class="label">Query Success Rate</div>
                <div class="value" id="querySuccessRate">0%</div>
            </div>
            <div class="stat-box">
                <div class="label">Avg Query Duration</div>
                <div class="value" id="avgQueryDuration">0ms</div>
            </div>
        </div>

        <div style="margin-bottom: 2rem;">
            <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Events by Type</h3>
            <div id="eventsByType" style="display: flex; gap: 1rem; flex-wrap: wrap;"></div>
        </div>

        <div style="margin-bottom: 2rem;">
            <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Most Common Queries</h3>
            <div id="commonQueries" style="display: flex; flex-direction: column; gap: 0.5rem;"></div>
        </div>

        <div style="margin-bottom: 2rem;">
            <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Most Accessed Memories</h3>
            <div id="accessedMemories" style="display: flex; flex-direction: column; gap: 0.5rem;"></div>
        </div>

        <script>
            async function loadAnalytics() {
                const days = document.getElementById('timeRange').value;
                
                try {
                    const resp = await fetch(`/api/history/analytics?days=${days}`);
                    const data = await resp.json();
                    
                    // Update stats
                    document.getElementById('totalEvents').innerText = data.total_events || 0;
                    document.getElementById('querySuccessRate').innerText = `${(data.query_success_rate * 100).toFixed(1)}%`;
                    document.getElementById('avgQueryDuration').innerText = `${data.average_query_duration_ms.toFixed(1)}ms`;
                    
                    // Events by type
                    const eventsByType = document.getElementById('eventsByType');
                    eventsByType.innerHTML = Object.entries(data.events_by_type || {}).map(([type, count]) => `
                        <div style="background: #0a0a0a; padding: 0.75rem; border-radius: 6px; border: 1px solid #222; min-width: 150px;">
                            <div style="font-size: 0.75rem; color: #666; text-transform: uppercase;">${type.replace('_', ' ')}</div>
                            <div style="font-size: 1.25rem; font-weight: 700; color: #fff;">${count}</div>
                        </div>
                    `).join('');
                    
                    // Most common queries
                    const commonQueries = document.getElementById('commonQueries');
                    if (data.most_common_queries && data.most_common_queries.length > 0) {
                        commonQueries.innerHTML = data.most_common_queries.map((q, i) => `
                            <div style="background: #0a0a0a; padding: 0.75rem; border-radius: 6px; border: 1px solid #222; display: flex; justify-content: space-between; align-items: center;">
                                <div style="flex: 1; margin-right: 1rem;">
                                    <div style="font-size: 0.9rem; color: #eee; line-height: 1.4;">${q.query || 'N/A'}</div>
                                </div>
                                <div style="font-size: 0.75rem; color: #666;">${q.count} queries</div>
                            </div>
                        `).join('');
                    } else {
                        commonQueries.innerHTML = '<div style="color: #444; text-align: center; padding: 1rem;">No queries found.</div>';
                    }
                    
                    // Most accessed memories
                    const accessedMemories = document.getElementById('accessedMemories');
                    if (data.most_accessed_memories && data.most_accessed_memories.length > 0) {
                        accessedMemories.innerHTML = data.most_accessed_memories.map((m, i) => `
                            <div style="background: #0a0a0a; padding: 0.75rem; border-radius: 6px; border: 1px solid #222; display: flex; justify-content: space-between; align-items: center;">
                                <div style="flex: 1; margin-right: 1rem;">
                                    <div style="font-size: 0.9rem; color: #eee; line-height: 1.4;">${m.content || 'N/A'}</div>
                                </div>
                                <div style="font-size: 0.75rem; color: #666;">${m.access_count} accesses</div>
                            </div>
                        `).join('');
                    } else {
                        accessedMemories.innerHTML = '<div style="color: #444; text-align: center; padding: 1rem;">No memories found.</div>';
                    }
                    
                } catch (e) {
                    console.error("Load analytics failed", e);
                }
            }
            
            loadAnalytics();
        </script>
    "#;
    
    Html(content.to_string())
}

/// Event details page
async fn event_details_page(
    State((_brain, _user_config)): State<(Arc<Brain>, Arc<tokio::sync::RwLock<UserConfig>>)>,
    Path(id): Path<String>,
) -> Html<String> {
    let content = format!(r#"
        <h2>Event Details</h2>
        <p style="color: #888; margin-bottom: 2rem;">Detailed information about this event.</p>
        
        <div id="eventDetails" style="display: flex; flex-direction: column; gap: 1.5rem;">
            <div style="text-align: center; padding: 2rem; color: #888;">Loading event details...</div>
        </div>

        <script>
            async function loadEventDetails() {{
                try {{
                    const resp = await fetch('/api/history/{}');
                    const data = await resp.json();
                    
                    const container = document.getElementById('eventDetails');
                    const timestamp = new Date(data.event.timestamp).toLocaleString();
                    const eventType = data.event.event_type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase());
                    
                    let details = '';
                    if (data.event.event_type === 'memory_created') {{
                        details = `
                            <div style="background: #0a0a0a; padding: 1.5rem; border-radius: 10px; border: 1px solid #222;">
                                <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Memory Information</h3>
                                <div style="margin-bottom: 1rem;">
                                    <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Content</div>
                                    <div style="font-size: 0.95rem; color: #eee; line-height: 1.5;">${{data.event.metadata.content || 'N/A'}}</div>
                                </div>
                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-top: 1rem;">
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Category</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.category || 'N/A'}}</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Importance</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.importance || 'N/A'}}</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Memory Type</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.memory_type || 'N/A'}}</div>
                                    </div>
                                </div>
                            </div>
                        `;
                    }} else if (data.event.event_type === 'query_searched') {{
                        details = `
                            <div style="background: #0a0a0a; padding: 1.5rem; border-radius: 10px; border: 1px solid #222;">
                                <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Query Information</h3>
                                <div style="margin-bottom: 1rem;">
                                    <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Query</div>
                                    <div style="font-size: 0.95rem; color: #eee; line-height: 1.5;">"${{data.event.metadata.query || 'N/A'}}"</div>
                                </div>
                                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-top: 1rem;">
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Results Count</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.results_count || 0}}</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Duration</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.duration_ms || 0}}ms</div>
                                    </div>
                                    <div>
                                        <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Limit</div>
                                        <div style="font-size: 0.9rem; color: #eee;">${{data.event.metadata.limit || 0}}</div>
                                    </div>
                                </div>
                            </div>
                        `;
                    }}
                    
                    let queryResults = '';
                    if (data.query_results && data.query_results.length > 0) {{
                        queryResults = `
                            <div style="background: #0a0a0a; padding: 1.5rem; border-radius: 10px; border: 1px solid #222;">
                                <h3 style="margin-bottom: 1rem; font-size: 0.9rem; color: #888;">Query Results</h3>
                                <div style="display: flex; flex-direction: column; gap: 0.75rem;">
                                    ${{data.query_results.map((result, i) => `
                                        <div style="background: #111; padding: 0.75rem; border-radius: 6px; display: flex; justify-content: space-between; align-items: center;">
                                            <div style="flex: 1; margin-right: 1rem;">
                                                <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Rank #${{result.rank}}</div>
                                                <div style="font-size: 0.85rem; color: #eee;">Memory ID: ${{result.memory_id}}</div>
                                            </div>
                                            <div style="font-size: 0.75rem; color: #666;">
                                                ${{result.similarity_score ? 'Similarity: ' + result.similarity_score.toFixed(3) : ''}}
                                                ${{result.recall_score ? ' | Recall: ' + result.recall_score.toFixed(3) : ''}}
                                            </div>
                                        </div>
                                    `).join('')}}
                                </div>
                            </div>
                        `;
                    }}
                    
                    container.innerHTML = `
                        <div style="background: #0a0a0a; padding: 1.5rem; border-radius: 10px; border: 1px solid #222;">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem;">
                                <div>
                                    <div style="font-size: 0.65rem; background: #1a1a1a; color: #888; padding: 0.2rem 0.5rem; border-radius: 4px; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.5rem;">${{eventType}}</div>
                                    <div style="font-size: 0.75rem; color: #666;">Event ID: ${{data.event.id}}</div>
                                </div>
                                <div style="font-size: 0.75rem; color: #444;">${{timestamp}}</div>
                            </div>
                            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem;">
                                <div>
                                    <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Namespace</div>
                                    <div style="font-size: 0.9rem; color: #eee;">${{data.event.namespace || 'N/A'}}</div>
                                </div>
                                <div>
                                    <div style="font-size: 0.75rem; color: #666; margin-bottom: 0.25rem;">Session ID</div>
                                    <div style="font-size: 0.9rem; color: #eee;">${{data.event.session_id || 'N/A'}}</div>
                                </div>
                            </div>
                        </div>
                        ${{details}}
                        ${{queryResults}}
                    `;
                    
                }} catch (e) {{
                    console.error("Load event details failed", e);
                    document.getElementById('eventDetails').innerHTML = '<div style="text-align: center; padding: 2rem; color: #888;">Failed to load event details.</div>';
                }}
            }}
            
            loadEventDetails();
        </script>
    "#, id);
    
    Html(content.to_string())
}