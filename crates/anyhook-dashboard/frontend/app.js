document.addEventListener('DOMContentLoaded', () => {
    // Navigation
    const navDashboard = document.getElementById('nav-dashboard');
    const navLogs = document.getElementById('nav-logs');
    const viewDashboard = document.getElementById('view-dashboard');
    const viewLogs = document.getElementById('view-logs');

    navDashboard.addEventListener('click', (e) => {
        e.preventDefault();
        navDashboard.classList.add('active');
        navLogs.classList.remove('active');
        viewDashboard.classList.add('active');
        viewLogs.classList.remove('active');
    });

    navLogs.addEventListener('click', (e) => {
        e.preventDefault();
        navLogs.classList.add('active');
        navDashboard.classList.remove('active');
        viewLogs.classList.add('active');
        viewDashboard.classList.remove('active');
        fetchLogs();
    });

    // Initial Fetch
    fetchStatus();

    // Auto refresh every 10 seconds
    setInterval(() => {
        if (viewDashboard.classList.contains('active')) {
            fetchStatus();
        }
    }, 10000);
});

async function fetchStatus() {
    try {
        const response = await fetch('/api/status');
        if (!response.ok) throw new Error('Failed to fetch status');
        const data = await response.json();
        
        // Update metrics
        document.getElementById('count-watchers').textContent = data.watchers.length;
        document.getElementById('count-hooks').textContent = data.hooks.length;
        
        let actionCount = 0;
        data.hooks.forEach(h => {
            actionCount += h.actions.length;
        });
        document.getElementById('count-actions').textContent = actionCount;

        // Render Watchers
        const watchersList = document.getElementById('watchers-list');
        watchersList.innerHTML = '';
        if (data.watchers.length === 0) {
            watchersList.innerHTML = '<li class="loading-state">No watchers configured</li>';
        } else {
            data.watchers.forEach(w => {
                const li = document.createElement('li');
                li.innerHTML = `
                    <div class="item-title" style="display: flex; justify-content: space-between; width: 100%; align-items: center;">
                        <div>
                            <span>${w.name}</span>
                            <span class="tag">${w.type}</span>
                        </div>
                        <button class="primary-btn" style="padding: 4px 12px; font-size: 0.8rem;" onclick="triggerWatcher('${w.name}')">▶ Trigger</button>
                    </div>
                `;
                watchersList.appendChild(li);
            });
        }

        // Render Hooks
        const hooksList = document.getElementById('hooks-list');
        hooksList.innerHTML = '';
        if (data.hooks.length === 0) {
            hooksList.innerHTML = '<li class="loading-state">No hooks configured</li>';
        } else {
            data.hooks.forEach(h => {
                const actionNames = h.actions.map(a => a.name || a.type).join(', ');
                const li = document.createElement('li');
                li.innerHTML = `
                    <div class="item-title">
                        <span>Trigger: ${h.watcher}</span>
                    </div>
                    <div class="item-subtitle">
                        Actions: <strong>${actionNames}</strong>
                    </div>
                `;
                hooksList.appendChild(li);
            });
        }
    } catch (error) {
        console.error('Error fetching status:', error);
    }
}

window.fetchLogs = async function() {
    const tbody = document.getElementById('logs-tbody');
    try {
        const response = await fetch('/api/logs');
        if (!response.ok) throw new Error('Failed to fetch logs');
        const logs = await response.json();
        
        tbody.innerHTML = '';
        if (logs.length === 0) {
            tbody.innerHTML = '<tr><td colspan="4" class="loading-state">No execution logs found</td></tr>';
            return;
        }

        logs.forEach(log => {
            const tr = document.createElement('tr');
            const statusClass = log.status.toLowerCase() === 'success' ? 'success' : 'failed';
            
            // Format SQLite UTC timestamp to local timezone
            let localTime = log.timestamp;
            try {
                let dateStr = log.timestamp;
                if (!dateStr.endsWith('Z')) {
                    dateStr = dateStr.replace(' ', 'T') + 'Z';
                }
                const d = new Date(dateStr);
                if (!isNaN(d)) {
                    localTime = d.toLocaleString();
                }
            } catch (e) {
                console.error('Time parse error:', e);
            }

            tr.innerHTML = `
                <td>#${log.id}</td>
                <td>${localTime}</td>
                <td><strong>${log.action_name}</strong></td>
                <td><span class="status-badge ${statusClass}">${log.status}</span></td>
            `;
            tbody.appendChild(tr);
        });
    } catch (error) {
        console.error('Error fetching logs:', error);
        tbody.innerHTML = '<tr><td colspan="4" class="loading-state" style="color:var(--error)">Error loading logs</td></tr>';
    }
};

window.clearLogs = async () => {
    if (!confirm("Are you sure you want to clear all execution logs? This cannot be undone.")) {
        return;
    }
    
    try {
        const response = await fetch('/api/logs', {
            method: 'DELETE'
        });
        
        if (response.ok) {
            fetchLogs(); // Reload empty list
        } else {
            console.error('Failed to clear logs');
            alert('Failed to clear logs');
        }
    } catch (error) {
        console.error('Error clearing logs:', error);
        alert('Error clearing logs');
    }
};

window.triggerWatcher = async (watcherName) => {
    try {
        const response = await fetch('/api/trigger', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ watcher: watcherName, payload: { source: 'dashboard' } })
        });
        
        if (response.ok) {
            alert(`Trigger signal sent to watcher: ${watcherName}! Check the Action Logs.`);
        } else {
            console.error('Failed to trigger');
            alert('Failed to trigger watcher');
        }
    } catch (error) {
        console.error('Error triggering watcher:', error);
        alert('Error triggering watcher');
    }
};
