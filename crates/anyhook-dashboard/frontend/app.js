document.addEventListener('DOMContentLoaded', () => {
    // Navigation
    const navDashboard = document.getElementById('nav-dashboard');
    const navLogs = document.getElementById('nav-logs');
    const navPlugins = document.getElementById('nav-plugins');
    const viewDashboard = document.getElementById('view-dashboard');
    const viewLogs = document.getElementById('view-logs');
    const viewPlugins = document.getElementById('view-plugins');

    function setActiveView(nav, view, fetchCb) {
        [navDashboard, navLogs, navPlugins].forEach(n => n.classList.remove('active'));
        [viewDashboard, viewLogs, viewPlugins].forEach(v => v.classList.remove('active'));
        nav.classList.add('active');
        view.classList.add('active');
        if (fetchCb) fetchCb();
    }

    navDashboard.addEventListener('click', (e) => {
        e.preventDefault();
        setActiveView(navDashboard, viewDashboard, null);
    });

    navLogs.addEventListener('click', (e) => {
        e.preventDefault();
        setActiveView(navLogs, viewLogs, fetchLogs);
    });

    navPlugins.addEventListener('click', (e) => {
        e.preventDefault();
        setActiveView(navPlugins, viewPlugins, fetchPlugins);
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

// i18n Logic
const i18n = {
    en: {
        navDashboard: 'Dashboard',
        navPlugins: 'Plugins',
        navLogs: 'Action Logs',
        engineOnline: 'Engine Online',
        activeWatchers: 'Active Watchers',
        registeredActions: 'Registered Actions',
        configuredHooks: 'Configured Hooks',
        watchers: 'Watchers',
        loadingWatchers: 'Loading watchers...',
        hooksAndActions: 'Hooks & Actions',
        loadingHooks: 'Loading hooks...',
        actionExecutionLogs: 'Action Execution Logs',
        clearLogs: 'Clear Logs',
        refresh: 'Refresh',
        id: 'ID',
        time: 'Time',
        action: 'Action',
        status: 'Status',
        loadingLogs: 'Loading logs...',
        installedPlugins: 'Installed Plugins',
        loadingPlugins: 'Loading plugins...',
        pluginDocTitle: 'Plugin Documentation',
        selectPluginPrompt: 'Select a plugin from the list to view its documentation.',
        triggerBtn: '▶ Trigger',
        noWatchers: 'No watchers configured',
        noHooks: 'No hooks configured',
        triggerPrefix: 'Trigger: ',
        actionsPrefix: 'Actions: ',
        watcherType: 'Type: ',
        clickToToggle: '(Click to toggle details)',
        noLogs: 'No execution logs found',
        errorLogs: 'Error loading logs',
        confirmClear: 'Are you sure you want to clear all execution logs? This cannot be undone.',
        failedClear: 'Failed to clear logs',
        errorClear: 'Error clearing logs',
        triggerSuccess: 'Trigger signal sent to watcher: {watcher}! Check the Action Logs.',
        triggerFailed: 'Failed to trigger watcher',
        triggerError: 'Error triggering watcher',
        noPlugins: 'No plugins found',
        errorPlugins: 'Error loading plugins',
        configured: 'Configured',
        autoDiscovered: 'Auto-discovered',
        docTitle: 'Documentation: {name}',
        noDocTitle: 'No Documentation Found',
        noDocMsg1: 'This plugin does not provide a <code>{name}.md</code> file in the plugins directory.',
        noDocMsg2: 'According to the Plugin Specification, all plugins MUST have a complete README/documentation.',
        configErrorTitle: 'Configuration Error (Reload Failed)',
        noBoundActions: 'No actions bound to this watcher'
    },
    zh: {
        navDashboard: '仪表盘',
        navPlugins: '插件',
        navLogs: '执行日志',
        engineOnline: '引擎运行中',
        activeWatchers: '活跃监听器',
        registeredActions: '注册动作',
        configuredHooks: '已配路由',
        watchers: '监听器 (Watchers)',
        loadingWatchers: '加载监听器...',
        hooksAndActions: '路由与动作 (Hooks & Actions)',
        loadingHooks: '加载路由...',
        actionExecutionLogs: '动作执行日志',
        clearLogs: '清空日志',
        refresh: '刷新',
        id: 'ID',
        time: '时间',
        action: '动作',
        status: '状态',
        loadingLogs: '加载日志...',
        installedPlugins: '已安装插件',
        loadingPlugins: '加载插件...',
        pluginDocTitle: '插件文档',
        selectPluginPrompt: '从列表中选择一个插件以查看其文档。',
        triggerBtn: '▶ 触发',
        noWatchers: '未配置任何监听器',
        noHooks: '未配置任何路由',
        triggerPrefix: '触发源: ',
        actionsPrefix: '动作: ',
        watcherType: '类型: ',
        clickToToggle: '(点击展开/折叠详情)',
        noLogs: '未找到执行日志',
        errorLogs: '加载日志失败',
        confirmClear: '您确定要清空所有执行日志吗？此操作不可撤销。',
        failedClear: '清空日志失败',
        errorClear: '清空日志时发生错误',
        triggerSuccess: '已向监听器发送触发信号: {watcher}! 请查看动作日志。',
        triggerFailed: '触发监听器失败',
        triggerError: '触发监听器时发生错误',
        noPlugins: '未找到插件',
        errorPlugins: '加载插件失败',
        configured: '已配置',
        autoDiscovered: '自动发现',
        docTitle: '文档: {name}',
        noDocTitle: '未找到文档',
        noDocMsg1: '此插件未在插件目录中提供 <code>{name}.md</code> 文件。',
        noDocMsg2: '根据插件规范，所有插件必须提供完整的自述/说明文档。',
        configErrorTitle: '配置文件错误 (热重载失败)',
        noBoundActions: '此监听器暂未绑定任何动作'
    }
};

let currentLang = localStorage.getItem('anyhook_lang') || 'en';

function applyI18n() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (i18n[currentLang][key]) {
            el.textContent = i18n[currentLang][key];
        }
    });
    document.getElementById('lang-selector').value = currentLang;
}

function t(key, vars = {}) {
    let str = i18n[currentLang][key] || key;
    for (let k in vars) {
        str = str.replace(`{${k}}`, vars[k]);
    }
    return str;
}

document.addEventListener('DOMContentLoaded', () => {
    applyI18n();
    document.getElementById('lang-selector').addEventListener('change', (e) => {
        currentLang = e.target.value;
        localStorage.setItem('anyhook_lang', currentLang);
        applyI18n();
        // Re-render dynamic lists
        if (document.getElementById('view-dashboard').classList.contains('active')) fetchStatus();
        if (document.getElementById('view-logs').classList.contains('active')) fetchLogs();
        if (document.getElementById('view-plugins').classList.contains('active')) fetchPlugins();
    });
});

async function fetchStatus() {
    try {
        const response = await fetch('/api/status');
        if (!response.ok) throw new Error('Failed to fetch status');
        const data = await response.json();
        
        // Handle Config Error
        const errorBanner = document.getElementById('config-error-banner');
        const errorMsg = document.getElementById('config-error-message');
        if (data.config_error) {
            errorMsg.textContent = data.config_error;
            errorBanner.style.display = 'block';
        } else {
            errorBanner.style.display = 'none';
        }

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
            watchersList.innerHTML = `<li class="loading-state">${t('noWatchers')}</li>`;
        } else {
            data.watchers.forEach((w, index) => {
                const boundHooks = data.hooks.filter(h => h.watcher === w.name);
                let detailsHtml = `<div id="watcher-details-${index}" style="display: none; margin-top: 1rem; padding-top: 1rem; border-top: 1px solid rgba(255,255,255,0.1);">`;
                
                if (boundHooks.length > 0) {
                    boundHooks.forEach((h, hidx) => {
                        detailsHtml += `<div style="margin-bottom: 0.5rem;"><strong style="color:var(--primary)">Hook #${hidx + 1}</strong></div>`;
                        detailsHtml += `<ul style="list-style: none; padding-left: 1rem; margin: 0 0 1rem 0;">`;
                        h.actions.forEach(a => {
                            detailsHtml += `<li style="margin-bottom: 4px;">- Action <strong>${a.name || a.type}</strong> <span class="tag" style="font-size: 0.7rem;">${a.type}</span></li>`;
                        });
                        detailsHtml += `</ul>`;
                    });
                } else {
                    detailsHtml += `<div style="color: #888; font-style: italic;">${t('noBoundActions')}</div>`;
                }
                detailsHtml += `</div>`;

                const li = document.createElement('li');
                li.style.cursor = 'pointer';
                li.onclick = (e) => {
                    if (e.target.tagName.toLowerCase() === 'button') return;
                    const details = document.getElementById(`watcher-details-${index}`);
                    details.style.display = details.style.display === 'none' ? 'block' : 'none';
                };
                
                li.innerHTML = `
                    <div class="item-title" style="display: flex; justify-content: space-between; width: 100%; align-items: center;">
                        <div>
                            <span style="font-size: 1.1rem; font-weight: 500;">${w.name}</span>
                            <span class="tag">${w.type}</span>
                            <span style="font-size: 0.8rem; color: #888; margin-left: 10px;">${t('clickToToggle')}</span>
                        </div>
                        <button class="primary-btn" style="padding: 4px 12px; font-size: 0.8rem;" onclick="triggerWatcher('${w.name}')">${t('triggerBtn')}</button>
                    </div>
                    ${detailsHtml}
                `;
                watchersList.appendChild(li);
            });
        }
        
        // Render Registered Actions
        const actionsList = document.getElementById('registered-actions-list');
        actionsList.innerHTML = '';
        if (!data.registered_actions || data.registered_actions.length === 0) {
            actionsList.innerHTML = `<li class="loading-state">No actions registered</li>`;
        } else {
            data.registered_actions.sort().forEach(a => {
                const li = document.createElement('li');
                li.innerHTML = `<div class="item-title"><span>${a}</span></div>`;
                actionsList.appendChild(li);
            });
        }
        
        // Render Configured Hooks
        const hooksList = document.getElementById('configured-hooks-list');
        hooksList.innerHTML = '';
        if (data.hooks.length === 0) {
            hooksList.innerHTML = `<li class="loading-state">${t('noHooks')}</li>`;
        } else {
            data.hooks.forEach(h => {
                const actionNames = h.actions.map(a => a.name || a.type).join(', ');
                const li = document.createElement('li');
                li.innerHTML = `
                    <div class="item-title">
                        <span>${t('triggerPrefix')}${h.watcher}</span>
                    </div>
                    <div class="item-subtitle">
                        ${t('actionsPrefix')}<strong>${actionNames}</strong>
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
            tbody.innerHTML = `<tr><td colspan="4" class="loading-state">${t('noLogs')}</td></tr>`;
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
        tbody.innerHTML = `<tr><td colspan="4" class="loading-state" style="color:var(--error)">${t('errorLogs')}</td></tr>`;
    }
};

window.clearLogs = async () => {
    if (!confirm(t('confirmClear'))) {
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
            alert(t('failedClear'));
        }
    } catch (error) {
        console.error('Error clearing logs:', error);
        alert(t('errorClear'));
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
            alert(t('triggerSuccess', { watcher: watcherName }));
        } else {
            console.error('Failed to trigger');
            alert(t('triggerFailed'));
        }
    } catch (error) {
        console.error('Error triggering watcher:', error);
        alert(t('triggerError'));
    }
};

window.pluginsData = [];

window.fetchPlugins = async function() {
    const list = document.getElementById('plugins-list');
    try {
        const response = await fetch('/api/plugins');
        if (!response.ok) throw new Error('Failed to fetch plugins');
        window.pluginsData = await response.json();
        
        list.innerHTML = '';
        if (window.pluginsData.length === 0) {
            list.innerHTML = `<li class="loading-state">${t('noPlugins')}</li>`;
            return;
        }

        window.pluginsData.forEach((p, index) => {
            const li = document.createElement('li');
            li.style.cursor = 'pointer';
            li.onclick = () => showPluginDoc(index);
            const tag = p.is_configured ? `<span class="tag" style="background:var(--primary);color:var(--bg)">${t('configured')}</span>` : `<span class="tag">${t('autoDiscovered')}</span>`;
            li.innerHTML = `
                <div class="item-title">
                    <span>${p.name}</span>
                    ${tag}
                </div>
                <div class="item-subtitle">
                    ${p.path}
                </div>
            `;
            list.appendChild(li);
        });
    } catch (error) {
        console.error('Error fetching plugins:', error);
        list.innerHTML = `<li class="loading-state" style="color:var(--error)">${t('errorPlugins')}</li>`;
    }
};

window.showPluginDoc = function(index) {
    const p = window.pluginsData[index];
    const title = document.getElementById('plugin-doc-title');
    const content = document.getElementById('plugin-doc-content');
    
    title.textContent = t('docTitle', { name: p.name });
    
    if (p.markdown_doc) {
        if (window.marked) {
            content.innerHTML = window.marked.parse(p.markdown_doc);
        } else {
            content.innerHTML = `<pre style="white-space: pre-wrap; font-family: inherit;">${p.markdown_doc}</pre>`;
        }
    } else {
        content.innerHTML = `
            <div style="text-align: center; color: var(--danger); padding: 2rem;">
                <h3>${t('noDocTitle')}</h3>
                <p>${t('noDocMsg1', { name: p.name })}</p>
                <p>${t('noDocMsg2')}</p>
            </div>
        `;
    }
};
