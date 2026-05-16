//! Web UI 模板
//!
//! 内嵌HTML模板，使用HTMX+Alpine.js实现轻量交互。
//! 服务端渲染片段，前端只做展示和交互，逻辑全在API层。

/// 共享导航栏
pub fn nav_html(active: &str) -> String {
    let items = [
        ("dashboard", "/", "📊 Dashboard"),
        ("seals", "/ui/seals", "🔐 Seals"),
        ("unseal", "/ui/unseal", "🔓 Unseal"),
        ("capsule", "/ui/capsule", "⏳ Capsule"),
        ("chain", "/ui/chain", "⛓️ Chain"),
        ("account", "/ui/account", "👤 Account"),
    ];
    let links: Vec<String> = items
        .iter()
        .map(|(key, href, label)| {
            if *key == active {
                format!(
                    r##"<a href="{}" style="color:var(--accent);font-weight:600">{}</a>"##,
                    href, label
                )
            } else {
                format!(r##"<a href="{}">{}</a>"##, href, label)
            }
        })
        .collect();
    format!(
        r##"<nav>
        <span class="logo">🧡 Jiaodai</span>
        {}
    </nav>"##,
        links.join("\n        ")
    )
}

/// 共享CSS样式
pub fn style_css() -> &'static str {
    r##"
        :root { --bg: #0f172a; --card: #1e293b; --accent: #f97316; --accent2: #eab308; --text: #e2e8f0; --dim: #94a3b8; --border: #334155; --ok: #22c55e; --warn: #eab308; --err: #ef4444; }
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; }
        nav { background: var(--card); border-bottom: 1px solid var(--border); padding: 1rem 2rem; display: flex; align-items: center; gap: 2rem; position: sticky; top: 0; z-index: 10; }
        nav .logo { font-size: 1.25rem; font-weight: 700; color: var(--accent); }
        nav a { color: var(--dim); text-decoration: none; font-size: 0.9rem; transition: color 0.2s; }
        nav a:hover { color: var(--text); }
        .container { max-width: 1200px; margin: 2rem auto; padding: 0 2rem; }
        .card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 1.5rem; margin-bottom: 1rem; }
        .card h2 { font-size: 1.1rem; margin-bottom: 1rem; color: var(--accent); }
        .stats-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 1rem; }
        .stat { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 1.25rem; text-align: center; }
        .stat .value { font-size: 1.75rem; font-weight: 700; color: var(--accent); }
        .stat .label { font-size: 0.8rem; color: var(--dim); margin-top: 0.25rem; }
        .btn { display: inline-block; padding: 0.5rem 1rem; border-radius: 6px; border: none; cursor: pointer; font-size: 0.85rem; transition: all 0.2s; }
        .btn-primary { background: var(--accent); color: #fff; }
        .btn-primary:hover { opacity: 0.9; }
        .btn-secondary { background: var(--border); color: var(--text); }
        .btn-secondary:hover { background: #475569; }
        input, select, textarea { background: var(--bg); color: var(--text); border: 1px solid var(--border); border-radius: 6px; padding: 0.5rem 0.75rem; font-size: 0.9rem; width: 100%; }
        input:focus, select:focus, textarea:focus { outline: none; border-color: var(--accent); }
        label { display: block; font-size: 0.85rem; color: var(--dim); margin-bottom: 0.25rem; }
        .form-group { margin-bottom: 1rem; }
        table { width: 100%; border-collapse: collapse; }
        th, td { padding: 0.75rem; text-align: left; border-bottom: 1px solid var(--border); }
        th { color: var(--dim); font-size: 0.8rem; font-weight: 600; text-transform: uppercase; }
        .badge { display: inline-block; padding: 0.15rem 0.5rem; border-radius: 9999px; font-size: 0.75rem; font-weight: 600; }
        .badge-ok { background: rgba(34,197,94,0.15); color: var(--ok); }
        .badge-warn { background: rgba(234,179,8,0.15); color: var(--warn); }
        .badge-err { background: rgba(239,68,68,0.15); color: var(--err); }
        .badge-dim { background: rgba(148,163,184,0.15); color: var(--dim); }
    "##
}

/// HTML外壳
fn page_shell(title: &str, body: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Jiaodai - {title}</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script defer src="https://cdn.jsdelivr.net/npm/alpinejs@3.x.x/dist/cdn.min.js"></script>
    <style>{}</style>
</head>
<body>
    {}
    <div class="container">
        {}
    </div>
</body>
</html>"##,
        style_css(),
        nav_html(title_to_active(title)),
        body
    )
}

fn title_to_active(title: &str) -> &str {
    match title {
        "Dashboard" => "dashboard",
        "Seals" => "seals",
        "Unseal" => "unseal",
        "Capsule" => "capsule",
        "Chain" => "chain",
        "Account" => "account",
        _ => "dashboard",
    }
}

/// Dashboard页面
pub fn dashboard_page() -> String {
    page_shell("Dashboard", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">🧡 Jiaodai Dashboard</h1>
        <div class="stats-grid" x-data="dashboardData()" x-init="load()">
            <div class="stat">
                <div class="value" x-text="stats.total_seals">-</div>
                <div class="label">Total Seals</div>
            </div>
            <div class="stat">
                <div class="value" x-text="stats.active_seals">-</div>
                <div class="label">Active Seals</div>
            </div>
            <div class="stat">
                <div class="value" x-text="stats.unsealed">-</div>
                <div class="label">Unsealed</div>
            </div>
            <div class="stat">
                <div class="value" x-text="stats.capsules">-</div>
                <div class="label">Time Capsules</div>
            </div>
            <div class="stat">
                <div class="value" x-text="stats.chain_proofs">-</div>
                <div class="label">Chain Proofs</div>
            </div>
        </div>
        <div class="card" style="margin-top:1.5rem">
            <h2>Quick Actions</h2>
            <div style="display:flex;gap:0.75rem;flex-wrap:wrap">
                <a href="/ui/seals" class="btn btn-primary">🔐 Create Seal</a>
                <a href="/ui/unseal" class="btn btn-secondary">🔓 Unseal Tape</a>
                <a href="/ui/capsule" class="btn btn-secondary">⏳ Create Capsule</a>
            </div>
        </div>
        <script>
        function dashboardData() {
            return {
                stats: { total_seals: '-', active_seals: '-', unsealed: '-', capsules: '-', chain_proofs: '-' },
                async load() {
                    try {
                        const r = await fetch('/api/v1/health');
                        const d = await r.json();
                        this.stats = { total_seals: '0', active_seals: '0', unsealed: '0', capsules: '0', chain_proofs: '0' };
                    } catch(e) { console.error(e); }
                }
            }
        }
        </script>
    "##)
}

/// Seals管理页面
pub fn seals_page() -> String {
    page_shell("Seals", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">🔐 Sealed Tapes</h1>
        <div class="card">
            <h2>Create New Seal</h2>
            <form x-data="sealForm()" @submit.prevent="submit()">
                <div class="form-group">
                    <label>Content Type</label>
                    <select x-model="form.content_type">
                        <option value="text">Text</option>
                        <option value="image">Image</option>
                        <option value="file">File</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Trigger Condition</label>
                    <select x-model="form.trigger_type">
                        <option value="date">Date Trigger</option>
                        <option value="match">Mutual Match</option>
                        <option value="heartbeat">Heartbeat (Will)</option>
                    </select>
                </div>
                <div class="form-group" x-show="form.trigger_type === 'date'">
                    <label>Open Date</label>
                    <input type="datetime-local" x-model="form.open_date">
                </div>
                <div class="form-group">
                    <label>Viewers (comma-separated phones)</label>
                    <input type="text" x-model="form.viewers" placeholder="13800138000,13900139000">
                </div>
                <button type="submit" class="btn btn-primary">🔒 Seal Now</button>
                <span x-text="result" style="margin-left:1rem;color:var(--ok)"></span>
            </form>
        </div>
        <div class="card">
            <h2>Recent Seals</h2>
            <table>
                <thead><tr><th>Tape ID</th><th>Type</th><th>Status</th><th>Created</th><th>Actions</th></tr></thead>
                <tbody x-data="sealsList()" x-init="load()">
                    <template x-for="s in seals" :key="s.id">
                        <tr>
                            <td x-text="s.id"></td>
                            <td x-text="s.type"></td>
                            <td><span class="badge badge-ok" x-text="s.status"></span></td>
                            <td x-text="s.created"></td>
                            <td><a :href="'/ui/seals'" class="btn btn-secondary" style="padding:0.2rem 0.5rem;font-size:0.8rem">View</a></td>
                        </tr>
                    </template>
                </tbody>
            </table>
        </div>
        <script>
        function sealForm() {
            return {
                form: { content_type: 'text', trigger_type: 'date', open_date: '', viewers: '' },
                result: '',
                async submit() {
                    try {
                        const viewers = this.form.viewers.split(',').filter(v=>v.trim()).map(v=>({phone:v.trim()}));
                        const r = await fetch('/api/v1/seal', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({content_type:this.form.content_type,trigger_condition:{type:this.form.trigger_type,open_date:this.form.open_date},viewers})});
                        const d = await r.json();
                        this.result = 'Sealed! ID: ' + d.tape_id;
                    } catch(e) { this.result = 'Error: ' + e.message; }
                }
            }
        }
        function sealsList() {
            return {
                seals: [],
                async load() {
                    try {
                        const r = await fetch('/api/v1/chain/batch');
                        const d = await r.json();
                        this.seals = (d.tapes||[]).map(t=>({id:t.tape_id||'-',type:t.content_type||'-',status:'sealed',created:t.created_at||'-'}));
                    } catch(e) { this.seals = []; }
                }
            }
        }
        </script>
    "##)
}

/// Unseal页面
pub fn unseal_page() -> String {
    page_shell("Unseal", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">🔓 Unseal Tape</h1>
        <div class="card" x-data="unsealForm()">
            <h2>Enter Tape ID</h2>
            <form @submit.prevent="submit()">
                <div class="form-group">
                    <label>Tape ID</label>
                    <input type="text" x-model="tape_id" placeholder="Enter tape ID to unseal">
                </div>
                <div class="form-group">
                    <label>Identity Claim (optional)</label>
                    <input type="text" x-model="identity" placeholder="Phone number or identity">
                </div>
                <button type="submit" class="btn btn-primary">🔓 Attempt Unseal</button>
            </form>
            <div x-show="result" style="margin-top:1rem;padding:1rem;border-radius:6px" :style="result_err ? 'background:rgba(239,68,68,0.15)' : 'background:rgba(34,197,94,0.15)'" x-text="result"></div>
        </div>
        <div class="card">
            <h2>Check Match Status</h2>
            <form x-data="matchForm()" @submit.prevent="check()">
                <div class="form-group">
                    <label>Tape ID</label>
                    <input type="text" x-model="tape_id" placeholder="Enter tape ID">
                </div>
                <button type="submit" class="btn btn-secondary">🔍 Check Match</button>
            </form>
        </div>
        <script>
        function unsealForm() {
            return {
                tape_id: '', identity: '', result: '', result_err: false,
                async submit() {
                    try {
                        const r = await fetch('/api/v1/unseal/'+this.tape_id, {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({identity_claim:this.identity})});
                        const d = await r.json();
                        this.result_err = !d.unsealed;
                        this.result = d.unsealed ? 'Unsealed! Content: ' + JSON.stringify(d.content||{}).substring(0,200) : (d.message || 'Cannot unseal yet');
                    } catch(e) { this.result_err = true; this.result = e.message; }
                }
            }
        }
        function matchForm() {
            return {
                tape_id: '',
                async check() {
                    try {
                        const r = await fetch('/api/v1/match/check?tape_id='+this.tape_id);
                        const d = await r.json();
                        alert(JSON.stringify(d, null, 2));
                    } catch(e) { alert('Error: '+e.message); }
                }
            }
        }
        </script>
    "##)
}

/// Time Capsule页面
pub fn capsule_page() -> String {
    page_shell("Capsule", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">⏳ Time Capsules</h1>
        <div class="card">
            <h2>Create Time Capsule</h2>
            <form x-data="capsuleForm()" @submit.prevent="submit()">
                <div class="form-group">
                    <label>Open Date</label>
                    <input type="datetime-local" x-model="form.open_at">
                </div>
                <div class="form-group">
                    <label>Viewers (comma-separated)</label>
                    <input type="text" x-model="form.viewers" placeholder="phone numbers">
                </div>
                <div class="form-group">
                    <label>Timezone</label>
                    <select x-model="form.timezone">
                        <option value="Asia/Shanghai">Asia/Shanghai (CST)</option>
                        <option value="UTC">UTC</option>
                        <option value="America/New_York">America/New_York (EST)</option>
                    </select>
                </div>
                <button type="submit" class="btn btn-primary">⏳ Create Capsule</button>
                <span x-text="result" style="margin-left:1rem;color:var(--ok)"></span>
            </form>
        </div>
        <div class="card">
            <h2>My Capsules</h2>
            <table x-data="capsulesList()" x-init="load()">
                <thead><tr><th>Capsule ID</th><th>Open At</th><th>Status</th><th>Countdown</th></tr></thead>
                <tbody>
                    <template x-for="c in capsules" :key="c.id">
                        <tr>
                            <td x-text="c.id"></td>
                            <td x-text="c.open_at"></td>
                            <td><span class="badge" :class="c.open ? 'badge-ok' : 'badge-warn'" x-text="c.open ? 'Open' : 'Sealed'"></span></td>
                            <td x-text="c.countdown || '-'"></td>
                        </tr>
                    </template>
                </tbody>
            </table>
        </div>
        <script>
        function capsuleForm() {
            return {
                form: { open_at: '', viewers: '', timezone: 'Asia/Shanghai' }, result: '',
                async submit() {
                    try {
                        const viewers = this.form.viewers.split(',').filter(v=>v.trim()).map(v=>({phone:v.trim()}));
                        const r = await fetch('/api/v1/capsule/create', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({creator_id:'web',open_at:this.form.open_at,viewers,timezone:this.form.timezone})});
                        const d = await r.json();
                        this.result = 'Created! ID: ' + (d.capsule_id || d.tape_id || 'ok');
                    } catch(e) { this.result = 'Error: ' + e.message; }
                }
            }
        }
        function capsulesList() {
            return {
                capsules: [],
                async load() {
                    try {
                        const r = await fetch('/api/v1/capsule/list');
                        const d = await r.json();
                        this.capsules = (d.capsules||[]).map(c=>({id:c.capsule_id||c.tape_id||'-',open_at:c.open_at||'-',open:c.is_open||false,countdown:c.countdown||null}));
                    } catch(e) { this.capsules = []; }
                }
            }
        }
        </script>
    "##)
}

/// Chain验证页面
pub fn chain_page() -> String {
    page_shell("Chain", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">⛓️ Chain Verification</h1>
        <div class="card">
            <h2>Verify On-Chain Proof</h2>
            <form x-data="chainForm()" @submit.prevent="verify()">
                <div class="form-group">
                    <label>Tape ID</label>
                    <input type="text" x-model="tape_id" placeholder="Enter tape ID for chain verification">
                </div>
                <button type="submit" class="btn btn-primary">⛓️ Verify</button>
            </form>
            <div x-show="result" style="margin-top:1rem" x-text="result"></div>
        </div>
        <div class="card">
            <h2>Batch Operations</h2>
            <div style="display:flex;gap:0.75rem">
                <button class="btn btn-secondary" x-data @click="batchSubmit()">📤 Submit Pending Hashes</button>
                <button class="btn btn-secondary" x-data @click="batchStatus()">📊 Batch Status</button>
            </div>
        </div>
        <script>
        function chainForm() {
            return {
                tape_id: '', result: '',
                async verify() {
                    try {
                        const r = await fetch('/api/v1/chain/verify/'+this.tape_id);
                        const d = await r.json();
                        this.result = JSON.stringify(d, null, 2);
                    } catch(e) { this.result = 'Error: '+e.message; }
                }
            }
        }
        </script>
    "##)
}

/// Account页面
pub fn account_page() -> String {
    page_shell("Account", r##"
        <h1 style="margin-bottom:1.5rem;font-size:1.5rem">👤 Account</h1>
        <div class="card">
            <h2>Login / Register</h2>
            <form x-data="authForm()" @submit.prevent="login()">
                <div class="form-group">
                    <label>Phone Number</label>
                    <input type="tel" x-model="phone" placeholder="13800138000">
                </div>
                <div class="form-group">
                    <label>Verification Code</label>
                    <input type="text" x-model="code" placeholder="Enter verification code">
                </div>
                <div style="display:flex;gap:0.75rem">
                    <button type="submit" class="btn btn-primary">Login</button>
                    <button type="button" class="btn btn-secondary" @click="register()">Register</button>
                </div>
                <div x-show="result" style="margin-top:0.75rem;color:var(--ok)" x-text="result"></div>
            </form>
        </div>
        <div class="card">
            <h2>API Status</h2>
            <div class="stats-grid">
                <div class="stat">
                    <div class="value" style="font-size:1rem">✅</div>
                    <div class="label">Service: Running</div>
                </div>
                <div class="stat">
                    <div class="value" style="font-size:1rem">v0.1.0</div>
                    <div class="label">API Version</div>
                </div>
            </div>
        </div>
        <script>
        function authForm() {
            return {
                phone: '', code: '', result: '',
                async login() {
                    try {
                        const r = await fetch('/api/v1/account/login', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({phone:this.phone,verification_code:this.code})});
                        const d = await r.json();
                        this.result = d.message || 'Logged in';
                    } catch(e) { this.result = 'Error: '+e.message; }
                },
                async register() {
                    try {
                        const r = await fetch('/api/v1/account/register', {method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({phone:this.phone,verification_code:this.code})});
                        const d = await r.json();
                        this.result = d.message || 'Registered';
                    } catch(e) { this.result = 'Error: '+e.message; }
                }
            }
        }
        </script>
    "##)
}
