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

// ── reCAPTCHA v3 ──────────────────────────────────────────────────────────
window.onRecaptchaLoad = () => {
  if (!window.__RC_SITE_KEY__) { openDashboard(); return; }
  grecaptcha.ready(() => { $('enter-btn').disabled = false; });
};

$('enter-btn').addEventListener('click', async () => {
  $('enter-btn').disabled = true;
  $('gate-error').hidden = true;
  try {
    const token = await grecaptcha.execute(window.__RC_SITE_KEY__, { action: 'enter' });
    const res   = await fetch('/api/verify-captcha', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token }),
    });
    const json = await res.json();
    if (json.success) {
      openDashboard();
    } else {
      showGateErr('Verification failed — please try again.');
      $('enter-btn').disabled = false;
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

  $('m-auth').textContent     = `${overview.success_pct}% success rate`;
  $('m-commands').textContent = `${overview.cmd_sessions} sessions with input`;

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
     <td class="mono">${r.shasum ? `<span title="${esc(r.shasum)}">${esc(r.shasum.substring(0, 14))}…</span>` : '<span style="color:var(--fg-dim)">—</span>'}</td>`
  );

  // Hide downloads section if nothing to show
  if (!d.dl_log.length && !d.top_urls.length) {
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

