'use strict';

// ── Gruvbox palette ───────────────────────────────────────────────────────
const GBX = {
  red:    '#fb4934', dred:   '#cc241d',
  green:  '#b8bb26', dgrn:   '#98971a',
  yellow: '#fabd2f', dyell:  '#d79921',
  blue:   '#83a598', dblue:  '#458588',
  purple: '#d3869b', dpurp:  '#b16286',
  aqua:   '#8ec07c', daqua:  '#689d6a',
  orange: '#fe8019', dorg:   '#d65d0e',
  gray:   '#a89984',
};

const PALETTE = [
  GBX.blue, GBX.green, GBX.yellow, GBX.red,
  GBX.aqua, GBX.orange, GBX.purple, GBX.gray,
];

// ── Helpers ───────────────────────────────────────────────────────────────
const $ = id => document.getElementById(id);

function esc(s) {
  return String(s ?? '')
    .replace(/&/g,'&amp;').replace(/</g,'&lt;')
    .replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

function fmtTime(iso, window) {
  const d = new Date(iso);
  if (window === '1h' || window === '6h' || window === '24h')
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  if (window === '7d')
    return d.toLocaleDateString([], { weekday: 'short', month: 'short', day: 'numeric' });
  return d.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function fmtTs(iso) {
  const d = new Date(iso);
  return d.toLocaleString([], {
    month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', second: '2-digit',
  });
}

function fgColor() {
  return document.documentElement.dataset.theme === 'light' ? '#504945' : '#a89984';
}

function gridColor() {
  return document.documentElement.dataset.theme === 'light'
    ? 'rgba(100,90,80,.15)' : 'rgba(128,128,128,.1)';
}

// ── Theme ─────────────────────────────────────────────────────────────────
const root = document.documentElement;
root.dataset.theme = localStorage.getItem('theme') || 'dark';
syncThemeIcon(root.dataset.theme);

$('theme-btn').addEventListener('click', () => {
  const next = root.dataset.theme === 'dark' ? 'light' : 'dark';
  root.dataset.theme = next;
  localStorage.setItem('theme', next);
  syncThemeIcon(next);
  // Update chart label colours
  Object.values(activeCharts).forEach(c => {
    const s = c.options.scales;
    if (s) Object.values(s).forEach(ax => {
      if (ax.ticks)  ax.ticks.color = fgColor();
      if (ax.grid)   ax.grid.color  = gridColor();
    });
    c.update();
  });
});

function syncThemeIcon(t) {
  $('icon-moon').hidden = t !== 'dark';
  $('icon-sun').hidden  = t !== 'light';
}

// ── reCAPTCHA ─────────────────────────────────────────────────────────────
let captchaToken = null;

window.onRecaptchaLoad = () => {
  if (!window.__RC_SITE_KEY__) { openDashboard(); return; }
  grecaptcha.render('captcha-widget', {
    sitekey: window.__RC_SITE_KEY__,
    theme: root.dataset.theme === 'light' ? 'light' : 'dark',
    callback: tok => { captchaToken = tok; $('enter-btn').disabled = false; },
    'expired-callback': () => { captchaToken = null; $('enter-btn').disabled = true; },
  });
};

$('enter-btn').addEventListener('click', async () => {
  if (!captchaToken) return;
  $('enter-btn').disabled = true;
  $('gate-error').hidden = true;
  try {
    const res  = await fetch('/api/verify-captcha', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: captchaToken }),
    });
    const json = await res.json();
    if (json.success) {
      openDashboard();
    } else {
      showGateErr('Verification failed — please try again.');
      grecaptcha.reset(); captchaToken = null; $('enter-btn').disabled = true;
    }
  } catch {
    showGateErr('Network error — please try again.');
    $('enter-btn').disabled = false;
  }
});

function showGateErr(msg) { $('gate-error').textContent = msg; $('gate-error').hidden = false; }

function openDashboard() {
  const gate = $('gate');
  gate.style.transition = 'opacity .45s ease';
  gate.style.opacity = '0';
  setTimeout(() => {
    gate.hidden = true;
    $('app').hidden = false;
    $('app').style.opacity = '0';
    requestAnimationFrame(() => {
      $('app').style.transition = 'opacity .4s ease';
      $('app').style.opacity = '1';
    });
    loadStats();
  }, 450);
}

// ── Window tab selector ───────────────────────────────────────────────────
let currentWindow = '6h';
let activeCharts  = {};

document.querySelectorAll('.win-tab').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.win-tab').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    currentWindow = btn.dataset.w;
    loadStats();
  });
});

$('refresh-btn').addEventListener('click', () => loadStats());

// ── Activity log tabs ─────────────────────────────────────────────────────
document.querySelectorAll('.log-tab').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.log-tab').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.log-panel').forEach(p => p.hidden = true);
    btn.classList.add('active');
    $('tab-' + btn.dataset.tab).hidden = false;
  });
});

// ── Fetch ─────────────────────────────────────────────────────────────────
async function loadStats() {
  $('loading').hidden    = false;
  $('stats-root').hidden = true;

  try {
    const res  = await fetch(`/api/stats?window=${currentWindow}`);
    const data = await res.json();
    if (data.error) throw new Error(data.error);
    destroyCharts();
    renderAll(data);
    $('loading').hidden    = true;
    $('stats-root').hidden = false;
    $('last-updated').textContent = new Date().toLocaleTimeString();
  } catch (err) {
    $('loading').innerHTML =
      `<p style="color:var(--c-red);font-family:monospace">Error: ${esc(err.message)}</p>`;
  }
}

function destroyCharts() {
  Object.values(activeCharts).forEach(c => c.destroy());
  activeCharts = {};
}

// ── Render ────────────────────────────────────────────────────────────────
function renderAll(d) {
  const { overview } = d;

  // ── Cards ─────────────────────────────────────────────────────────────
  counter($('v-connections'), overview.connections);
  counter($('v-auth'),        overview.auth_attempts);
  counter($('v-commands'),    overview.commands);
  counter($('v-ips'),         overview.unique_ips);
  counter($('v-downloads'),   overview.downloads);
  counter($('v-malware'),     overview.malware_hashes);

  $('m-auth').textContent     = `${overview.success_pct}% success rate`;
  $('m-commands').textContent = `${overview.cmd_sessions} sessions with input`;
  $('m-downloads').textContent = `${overview.malware_hashes} unique hashes`;

  // ── Timeseries ────────────────────────────────────────────────────────
  if (d.timeseries.length) {
    const labels  = d.timeseries.map(r => fmtTime(r.t, currentWindow));
    const failed  = d.timeseries.map(r => r.failed);
    const success = d.timeseries.map(r => r.successful);
    activeCharts['ts'] = new Chart($('chart-timeseries'), {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: 'Failed',
            data: failed,
            borderColor: GBX.red, backgroundColor: GBX.red + '22',
            fill: true, tension: .35, pointRadius: 0, borderWidth: 2,
          },
          {
            label: 'Successful',
            data: success,
            borderColor: GBX.green, backgroundColor: GBX.green + '22',
            fill: true, tension: .35, pointRadius: 0, borderWidth: 2,
          },
        ],
      },
      options: {
        responsive: true, maintainAspectRatio: false,
        interaction: { mode: 'index', intersect: false },
        plugins: { legend: { display: false } },
        scales: {
          x: { ticks: { color: fgColor(), maxTicksLimit: 8 }, grid: { color: gridColor() } },
          y: { ticks: { color: fgColor(), stepSize: 1 },      grid: { color: gridColor() }, beginAtZero: true },
        },
      },
    });
  }

  // ── Hour of day ───────────────────────────────────────────────────────
  {
    const counts = new Array(24).fill(0);
    d.by_hour.forEach(r => { counts[r.h] = Number(r.attempts); });
    const labels = counts.map((_, i) => `${String(i).padStart(2,'0')}:00`);
    activeCharts['hour'] = barChart('chart-hour', labels, counts, GBX.blue + 'cc');
  }

  // ── Day of week ───────────────────────────────────────────────────────
  {
    const days = ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'];
    const counts = new Array(7).fill(0);
    d.by_dow.forEach(r => { counts[r.dow] = Number(r.attempts); });
    activeCharts['dow'] = barChart('chart-dow', days, counts, GBX.orange + 'cc');
  }

  // ── Top usernames ─────────────────────────────────────────────────────
  if (d.top_usernames.length) {
    const labels = d.top_usernames.map(r => r.username);
    const values = d.top_usernames.map(r => Number(r.attempts));
    activeCharts['users'] = hbarChart('chart-usernames', labels, values, GBX.aqua + 'cc');
  }

  // ── Top passwords ─────────────────────────────────────────────────────
  if (d.top_passwords.length) {
    const labels = d.top_passwords.map(r => r.password);
    const values = d.top_passwords.map(r => Number(r.attempts));
    activeCharts['pw'] = hbarChart('chart-passwords', labels, values, GBX.purple + 'cc');
  }

  // ── Top URLs ──────────────────────────────────────────────────────────
  if (d.top_urls.length) {
    const labels = d.top_urls.map(r => r.url.replace(/^https?:\/\//, '').substring(0, 40));
    const values = d.top_urls.map(r => Number(r.downloads));
    activeCharts['urls'] = hbarChart('chart-urls', labels, values, GBX.yellow + 'cc');
  }

  // ── Tables ────────────────────────────────────────────────────────────
  fillTable('tbl-pairs', d.top_pairs, r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono">${esc(r.username)}</td>
     <td class="mono">${esc(r.password)}</td>
     <td class="num mono">${r.attempts}</td>`
  );

  fillTable('tbl-clients', d.ssh_clients, r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono truncate">${esc(r.client_version)}</td>
     <td class="num mono">${r.connections}</td>`
  );

  fillTable('tbl-malware', d.malware_files, r =>
    `<td class="mono truncate">${esc(r.filename)}</td>
     <td class="mono">${hashCell(r.shasum)}</td>
     <td class="num mono">${r.downloads}</td>`
  );

  // ── Activity logs ─────────────────────────────────────────────────────
  fillTable('tbl-commands', d.cmd_log, r =>
    `<td class="mono">${fmtTs(r.time)}</td>
     <td class="mono">${esc(r.ip)}</td>
     <td class="mono">${esc(r.session)}</td>
     <td class="mono truncate">${esc(r.input)}</td>`
  );

  fillTable('tbl-auth', d.auth_log, r =>
    `<td class="mono">${fmtTs(r.time)}</td>
     <td class="mono">${esc(r.ip)}</td>
     <td class="mono">${esc(r.username)}</td>
     <td class="mono">${esc(r.password)}</td>
     <td class="mono ${r.success ? 'badge-ok' : 'badge-fail'}">${r.success ? '✓' : '✗'}</td>`
  );

  fillTable('tbl-dl', d.dl_log, r =>
    `<td class="mono">${fmtTs(r.time)}</td>
     <td class="mono">${esc(r.ip)}</td>
     <td class="mono truncate">${esc(r.url)}</td>
     <td class="mono">${hashCell(r.shasum)}</td>`
  );

  // Hide downloads section if nothing to show
  if (!d.dl_log.length && !d.malware_files.length && !d.top_urls.length) {
    $('section-downloads').hidden = true;
  }
}

// ── Counter animation ─────────────────────────────────────────────────────
function counter(el, target) {
  if (!el) return;
  if (!Number.isFinite(target) || target === 0) { if (el) el.textContent = target; return; }
  const step = Math.max(1, Math.ceil(target / (1100 / 16)));
  let cur = 0;
  const tick = () => {
    cur = Math.min(cur + step, target);
    el.textContent = cur.toLocaleString();
    if (cur < target) requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

// ── Chart helpers ─────────────────────────────────────────────────────────
function barChart(canvasId, labels, data, color) {
  return new Chart($(canvasId), {
    type: 'bar',
    data: { labels, datasets: [{ data, backgroundColor: color, borderRadius: 4, borderWidth: 0 }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      plugins: { legend: { display: false } },
      scales: {
        x: { ticks: { color: fgColor(), maxRotation: 0 }, grid: { color: gridColor() } },
        y: { ticks: { color: fgColor(), stepSize: 1 }, grid: { color: gridColor() }, beginAtZero: true },
      },
    },
  });
}

function hbarChart(canvasId, labels, data, color) {
  return new Chart($(canvasId), {
    type: 'bar',
    data: { labels, datasets: [{ data, backgroundColor: color, borderRadius: 4, borderWidth: 0 }] },
    options: {
      responsive: true, maintainAspectRatio: false,
      indexAxis: 'y',
      plugins: { legend: { display: false } },
      scales: {
        x: { ticks: { color: fgColor(), stepSize: 1 }, grid: { color: gridColor() }, beginAtZero: true },
        y: { ticks: { color: fgColor() }, grid: { display: false } },
      },
    },
  });
}

// ── Table filler ──────────────────────────────────────────────────────────
function fillTable(id, rows, rowHtml) {
  const tbl = $(id);
  if (!tbl) return;
  const tbody = tbl.querySelector('tbody');
  if (!rows.length) {
    tbody.innerHTML = `<tr><td colspan="10" style="text-align:center;color:var(--fg-dim);padding:20px">No data for this window</td></tr>`;
    return;
  }
  tbody.innerHTML = rows.map((r, i) => `<tr>${rowHtml({ ...r, _rank: i + 1 })}</tr>`).join('');
}

// ── Hash cell helper ──────────────────────────────────────────────────────
function hashCell(sha256) {
  if (!sha256) return '<span style="color:var(--fg-dim)">—</span>';
  const short = sha256.substring(0, 14) + '…';
  return `<span class="mono" title="${esc(sha256)}">${esc(short)}</span>`
    + `<button class="hash-btn" data-hash="${esc(sha256)}">`
    + `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">`
    + `<circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>`
    + `</svg> Analyze</button>`;
}

// ── Event delegation for hash buttons ────────────────────────────────────
document.addEventListener('click', e => {
  const btn = e.target.closest('.hash-btn');
  if (btn) openLookup(btn.dataset.hash);
});

// ── Modal ─────────────────────────────────────────────────────────────────
const modal      = $('lookup-modal');
const modalBody  = $('modal-body');
const modalTitle = $('modal-title');
const modalHash  = $('modal-hash');
const modalIcon  = $('modal-icon');

$('modal-close').addEventListener('click', closeLookup);
modal.addEventListener('click', e => { if (e.target === modal) closeLookup(); });
document.addEventListener('keydown', e => { if (e.key === 'Escape') closeLookup(); });

function closeLookup() {
  modal.hidden = true;
}

async function openLookup(sha256) {
  // Reset
  modalTitle.textContent = 'Hash Analysis';
  modalHash.textContent  = sha256;
  modalIcon.className    = 'modal-icon';
  modalBody.innerHTML    = `<div class="modal-loading"><div class="spinner"></div><p>Querying threat intelligence…</p></div>`;
  modal.hidden = false;
  modal.querySelector('.modal-card').focus?.();

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 20000);
  try {
    const res  = await fetch(`/api/lookup/${encodeURIComponent(sha256)}`, { signal: controller.signal });
    const data = await res.json();
    renderLookup(data);
  } catch (err) {
    const msg = err.name === 'AbortError' ? 'Request timed out' : err.message;
    modalBody.innerHTML = `<p style="color:var(--c-red);font-family:monospace;padding:8px">Error: ${esc(msg)}</p>`;
  } finally {
    clearTimeout(timer);
  }
}

function renderLookup(data) {
  const mb = data.malwarebazaar || {};
  const vt = data.virustotal    || null;

  // Determine icon / title
  const malwareName = mb.signature || vt?.name || (mb.found || vt?.found ? 'Malware detected' : 'Hash not found');
  const isFound     = mb.found || vt?.found;
  modalTitle.textContent = malwareName;
  modalIcon.className    = `modal-icon ${isFound ? 'found' : 'clean'}`;
  if (!isFound) {
    modalIcon.innerHTML = svgCheck();
    modalTitle.textContent = 'Not found in threat databases';
  } else {
    modalIcon.innerHTML = svgWarn();
  }

  const parts = [];

  // ── MalwareBazaar section ───────────────────────────────────────────────
  if (mb.found) {
    const kv = [
      mb.file_name    && ['Filename',   mb.file_name],
      mb.file_type    && ['File type',  `${mb.file_type}${mb.file_type_mime ? ' · ' + mb.file_type_mime : ''}`],
      mb.file_size    && ['Size',       fmtBytes(mb.file_size)],
      mb.first_seen   && ['First seen', mb.first_seen],
      mb.last_seen    && ['Last seen',  mb.last_seen],
      mb.reporter     && ['Reporter',   mb.reporter],
      mb.origin       && ['Origin',     mb.origin],
      mb.delivery     && ['Delivery',   mb.delivery],
      mb.downloads    && ['Downloads',  mb.downloads],
      mb.clamav       && ['ClamAV',     mb.clamav],
      mb.md5          && ['MD5',        mb.md5],
      mb.sha1         && ['SHA-1',      mb.sha1],
    ].filter(Boolean);

    parts.push(`
      <div class="modal-section">
        <div class="modal-section-title">
          MalwareBazaar <span class="source-badge source-mb">abuse.ch</span>
        </div>
        <div class="kv-grid">${kv.map(([k,v]) => `<span class="kv-key">${esc(k)}</span><span class="kv-val">${esc(v)}</span>`).join('')}</div>
        ${mb.tags?.length ? `<div class="tag-row">${mb.tags.map(t => `<span class="modal-tag">${esc(t)}</span>`).join('')}</div>` : ''}
        <div class="modal-links">
          <a class="ext-link mb" href="${esc(mb.url)}" target="_blank" rel="noopener">
            ${svgExternal()} View on MalwareBazaar
          </a>
        </div>
      </div>`);
  } else {
    parts.push(`
      <div class="modal-section">
        <div class="modal-section-title">MalwareBazaar <span class="source-badge source-mb">abuse.ch</span></div>
        <div class="not-found-note">Hash not found in MalwareBazaar${mb.error ? ' — ' + esc(mb.error) : ''}</div>
        <div class="modal-links">
          <a class="ext-link mb" href="https://bazaar.abuse.ch/sample/${esc(data.sha256)}/" target="_blank" rel="noopener">
            ${svgExternal()} Search on MalwareBazaar
          </a>
        </div>
      </div>`);
  }

  // ── VirusTotal section ──────────────────────────────────────────────────
  if (vt === null) {
    parts.push(`
      <div class="modal-section">
        <div class="modal-section-title">VirusTotal <span class="source-badge source-vt">optional</span></div>
        <div class="not-found-note">Set <code>VIRUSTOTAL_API_KEY</code> in <code>.env</code> to enable VirusTotal lookups</div>
      </div>`);
  } else if (vt.found) {
    const pct   = vt.total > 0 ? Math.round((vt.malicious + vt.suspicious) / vt.total * 100) : 0;
    const clr   = pct === 0 ? 'var(--c-green)' : pct < 20 ? 'var(--c-yellow)' : pct < 50 ? 'var(--c-orange)' : 'var(--c-red)';
    const allTags = [...new Set([...(vt.tags||[]), ...(vt.names||[])])].slice(0, 10);

    parts.push(`
      <div class="modal-section">
        <div class="modal-section-title">VirusTotal <span class="source-badge source-vt">virustotal.com</span></div>
        <div class="detection-wrap">
          <div class="detection-bar">
            <div class="detection-fill" style="width:${pct}%;background:${clr}"></div>
          </div>
          <span class="detection-label" style="color:${clr}">${vt.malicious + vt.suspicious} / ${vt.total} engines</span>
        </div>
        <div class="kv-grid">
          ${vt.type     ? `<span class="kv-key">Type</span><span class="kv-val">${esc(vt.type)}</span>` : ''}
          ${vt.size     ? `<span class="kv-key">Size</span><span class="kv-val">${fmtBytes(vt.size)}</span>` : ''}
          ${vt.first_seen ? `<span class="kv-key">First seen</span><span class="kv-val">${new Date(vt.first_seen*1000).toISOString().slice(0,10)}</span>` : ''}
          ${vt.last_seen  ? `<span class="kv-key">Last seen</span><span class="kv-val">${new Date(vt.last_seen*1000).toISOString().slice(0,10)}</span>` : ''}
          <span class="kv-key">Suspicious</span><span class="kv-val">${vt.suspicious}</span>
          <span class="kv-key">Undetected</span><span class="kv-val">${vt.undetected}</span>
        </div>
        ${allTags.length ? `<div class="tag-row">${allTags.map(t => `<span class="modal-tag vt-tag">${esc(t)}</span>`).join('')}</div>` : ''}
        <div class="modal-links">
          <a class="ext-link vt" href="${esc(vt.url)}" target="_blank" rel="noopener">
            ${svgExternal()} View on VirusTotal
          </a>
        </div>
      </div>`);
  } else {
    parts.push(`
      <div class="modal-section">
        <div class="modal-section-title">VirusTotal <span class="source-badge source-vt">virustotal.com</span></div>
        <div class="not-found-note">Hash not found in VirusTotal${vt.error ? ' — ' + esc(vt.error) : ''}</div>
        <div class="modal-links">
          <a class="ext-link vt" href="https://www.virustotal.com/gui/file/${esc(data.sha256)}" target="_blank" rel="noopener">
            ${svgExternal()} Search on VirusTotal
          </a>
        </div>
      </div>`);
  }

  modalBody.innerHTML = parts.join('');
}

// ── SVG snippets ──────────────────────────────────────────────────────────
function svgWarn() {
  return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>`;
}
function svgCheck() {
  return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>`;
}
function svgExternal() {
  return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>`;
}

// ── Format bytes ──────────────────────────────────────────────────────────
function fmtBytes(n) {
  if (n < 1024)       return `${n} B`;
  if (n < 1024**2)    return `${(n/1024).toFixed(1)} KB`;
  if (n < 1024**3)    return `${(n/1024**2).toFixed(1)} MB`;
  return `${(n/1024**3).toFixed(1)} GB`;
}
