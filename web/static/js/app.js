'use strict';

// ── Translations ──────────────────────────────────────────────────────────
const STRINGS = {
  en: {
    'gate.bar':            'autopot — access control',
    'gate.subtitle':       'automated honeypot — ssh / telnet',
    'gate.desc':           "Verify you're human to access live attack data.",
    'gate.btn':            '[ ENTER DASHBOARD ]',
    'gate.err.verify':     'Verification failed — please try again.',
    'gate.err.network':    'Network error — please try again.',

    'nav.dashboard':       'Dashboard',
    'nav.wordlists':       'Wordlists',
    'win.all':             'All',

    'card.connections':           'Connections',
    'card.connections.meta':      'distinct attacker sessions',
    'card.auth':                  'Auth Attempts',
    'card.commands':              'Commands Run',
    'card.ips':                   'Unique IPs',
    'card.ips.meta':              'distinct source addresses',
    'card.downloads':             'File Downloads',
    'card.unique_passwords':      'Unique Passwords',
    'card.unique_passwords.meta': 'distinct passwords collected',
    'card.unique_usernames':      'Unique Usernames',
    'card.unique_usernames.meta': 'distinct usernames collected',
    'card.malware_hashes':        'Malware Hashes',
    'card.malware_hashes.meta':   'distinct file hashes collected',

    'section.overview':        'Overview',
    'section.timeseries':      'Login Attempts Over Time',
    'section.timing':          'Timing Patterns',
    'section.credentials':     'Top Credentials',
    'section.password_hashes': 'Password Hashes',
    'section.pairs':           'Top Credential Pairs',
    'section.clients':         'SSH Client Versions',
    'section.downloads':       'Downloads',
    'section.malware_hashes':  'Malware Hashes',
    'section.logs':            'Activity Logs',

    'th.first_seen':   'First Seen',
    'th.downloads':    'Downloads',

    'chart.by_hour':   'Attacks by Hour of Day',
    'chart.by_dow':    'Attacks by Day of Week',
    'chart.usernames': 'Usernames',
    'chart.passwords': 'Passwords',
    'chart.top_urls':  'Top Download URLs',

    'legend.failed':     'Failed',
    'legend.successful': 'Successful',

    'th.rank':        '#',
    'th.username':    'Username',
    'th.password':    'Password',
    'th.sha256':      'SHA-256',
    'th.attempts':    'Attempts',
    'th.client':      'Client String',
    'th.connections': 'Connections',
    'th.time':        'Time',
    'th.ip':          'IP',
    'th.session':     'Session',
    'th.command':     'Command',
    'th.ok':          'OK',
    'th.url':         'URL',
    'th.sha':         'SHA-256',

    'tab.commands': 'Commands',
    'tab.auth':     'Auth Log',
    'tab.dl':       'Downloads',

    'loading.stats':      'Loading statistics…',
    'loading.stats.spin': 'loading statistics…',
    'loading.wordlists':  'Generating wordlists…',
    'tbl.nodata':         'No data for this window',

    'wl.title':         'Generated Wordlists',
    'wl.desc':          'Ranked by attack frequency — most-seen credentials first. Downloads are gzip-compressed plaintext.',
    'wl.period.daily':   'Daily',
    'wl.period.weekly':  'Weekly',
    'wl.period.monthly': 'Monthly',
    'wl.period.all':     'All Time',
    'wl.usernames':     'Usernames',
    'wl.passwords':     'Passwords',
    'wl.pairs':         'Credential Pairs',
    'wl.entries':       'entries',
    'wl.period':        'period',
    'wl.size':          '~size (gz)',
    'wl.preview':       'preview — top 5 by frequency',
    'wl.preview.pairs': 'preview — top 5 by frequency (user:pass format)',
    'wl.empty':         'no data collected yet',

    'meta.success_pct':  n => `${n}% success rate`,
    'meta.cmd_sessions': n => `${n} sessions with input`,
    'period.nodata':     'no data',

    'dow': ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'],
  },
  pt: {
    'gate.bar':            'autopot — controle de acesso',
    'gate.subtitle':       'honeypot automatizado — ssh / telnet',
    'gate.desc':           'Verifique que você é humano para acessar os dados de ataque em tempo real.',
    'gate.btn':            '[ ENTRAR NO PAINEL ]',
    'gate.err.verify':     'Verificação falhou — tente novamente.',
    'gate.err.network':    'Erro de rede — tente novamente.',

    'nav.dashboard':       'Painel',
    'nav.wordlists':       'Wordlists',
    'win.all':             'Tudo',

    'card.connections':           'Conexões',
    'card.connections.meta':      'sessões de atacantes distintos',
    'card.auth':                  'Tentativas de Login',
    'card.commands':              'Comandos Executados',
    'card.ips':                   'IPs Únicos',
    'card.ips.meta':              'endereços de origem distintos',
    'card.downloads':             'Arquivos Baixados',
    'card.unique_passwords':      'Senhas Coletadas',
    'card.unique_passwords.meta': 'senhas distintas coletadas',
    'card.unique_usernames':      'Usuários Coletados',
    'card.unique_usernames.meta': 'usuários distintos coletados',
    'card.malware_hashes':        'Hashes de Malware',
    'card.malware_hashes.meta':   'hashes de arquivos coletados',

    'section.overview':        'Visão Geral',
    'section.timeseries':      'Tentativas de Login ao Longo do Tempo',
    'section.timing':          'Padrões Temporais',
    'section.credentials':     'Credenciais Mais Usadas',
    'section.password_hashes': 'Hashes de Senhas',
    'section.pairs':           'Pares de Credenciais Mais Usados',
    'section.clients':         'Versões de Cliente SSH',
    'section.downloads':       'Downloads',
    'section.malware_hashes':  'Hashes de Malware',
    'section.logs':            'Registros de Atividade',

    'th.first_seen':   'Primeira Vez',
    'th.downloads':    'Downloads',

    'chart.by_hour':   'Ataques por Hora do Dia',
    'chart.by_dow':    'Ataques por Dia da Semana',
    'chart.usernames': 'Usuários',
    'chart.passwords': 'Senhas',
    'chart.top_urls':  'URLs Mais Baixadas',

    'legend.failed':     'Falhou',
    'legend.successful': 'Bem-sucedido',

    'th.rank':        '#',
    'th.username':    'Usuário',
    'th.password':    'Senha',
    'th.sha256':      'SHA-256',
    'th.attempts':    'Tentativas',
    'th.client':      'Versão do Cliente',
    'th.connections': 'Conexões',
    'th.time':        'Hora',
    'th.ip':          'IP',
    'th.session':     'Sessão',
    'th.command':     'Comando',
    'th.ok':          'OK',
    'th.url':         'URL',
    'th.sha':         'SHA-256',

    'tab.commands': 'Comandos',
    'tab.auth':     'Log de Auth',
    'tab.dl':       'Downloads',

    'loading.stats':      'Carregando estatísticas…',
    'loading.stats.spin': 'carregando estatísticas…',
    'loading.wordlists':  'Gerando wordlists…',
    'tbl.nodata':         'Sem dados para este período',

    'wl.title':         'Wordlists Geradas',
    'wl.desc':          'Ordenadas por frequência de ataque — credenciais mais vistas primeiro. Os downloads são texto comprimido com gzip.',
    'wl.period.daily':   'Diário',
    'wl.period.weekly':  'Semanal',
    'wl.period.monthly': 'Mensal',
    'wl.period.all':     'Tudo',
    'wl.usernames':     'Usuários',
    'wl.passwords':     'Senhas',
    'wl.pairs':         'Pares de Credenciais',
    'wl.entries':       'entradas',
    'wl.period':        'período',
    'wl.size':          '~tamanho (gz)',
    'wl.preview':       'prévia — top 20 por frequência',
    'wl.preview.pairs': 'prévia — top 20 por frequência (formato usuário:senha)',
    'wl.empty':         'nenhum dado coletado ainda',

    'meta.success_pct':  n => `${n}% taxa de sucesso`,
    'meta.cmd_sessions': n => `${n} sessões com comandos`,
    'period.nodata':     'sem dados',

    'dow': ['Dom','Seg','Ter','Qua','Qui','Sex','Sáb'],
  },
};

let currentLang = localStorage.getItem('lang') || 'en';

function t(key) {
  return (STRINGS[currentLang] || STRINGS.en)[key] ?? STRINGS.en[key] ?? key;
}

function tf(key, val) {
  const v = t(key);
  return typeof v === 'function' ? v(val) : v;
}

function applyLang(lang) {
  currentLang = lang;
  document.documentElement.lang = lang;
  localStorage.setItem('lang', lang);

  document.querySelectorAll('[data-i18n]').forEach(el => {
    const val = t(el.dataset.i18n);
    if (typeof val === 'string') el.textContent = val;
  });

  document.querySelectorAll('.lang-tab').forEach(b => {
    b.classList.toggle('active', b.dataset.lang === lang);
  });

  // Re-render live data with new language (translated DOW labels, empty states, etc.)
  if (lastData && !$('page-dashboard').hidden) {
    destroyCharts();
    renderAll(lastData);
  }
  if (lastWlData && !$('page-wordlists').hidden) {
    renderWordlists(lastWlData);
  }
}

// ── Gruvbox chart palette (bright — readable on light and dark) ───────────
const GBX = {
  red:    '#FB4934',
  green:  '#B8BB26',
  yellow: '#FABD2F',
  blue:   '#83A598',
  purple: '#D3869B',
  aqua:   '#8EC07C',
  orange: '#FE8019',
  gray:   '#A89984',
};

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

function fmtPeriod(oldest, newest) {
  if (!oldest) return t('period.nodata');
  const fmt = iso => new Date(iso).toLocaleDateString([], { month: 'short', day: 'numeric', year: '2-digit' });
  return `${fmt(oldest)} – ${fmt(newest)}`;
}

function fgColor() {
  return document.documentElement.dataset.theme === 'dark' ? '#A89984' : '#665C54';
}

function gridColor() {
  return document.documentElement.dataset.theme === 'dark'
    ? 'rgba(235,219,178,.06)' : 'rgba(60,56,54,.07)';
}

// ── Theme ─────────────────────────────────────────────────────────────────
const root = document.documentElement;
root.dataset.theme = localStorage.getItem('theme') || 'light';
syncThemeIcon(root.dataset.theme);

document.querySelectorAll('.theme-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    const next = root.dataset.theme === 'dark' ? 'light' : 'dark';
    root.dataset.theme = next;
    localStorage.setItem('theme', next);
    syncThemeIcon(next);
    Object.values(activeCharts).forEach(c => {
      const s = c.options.scales;
      if (s) Object.values(s).forEach(ax => {
        if (ax.ticks) ax.ticks.color = fgColor();
        if (ax.grid)  ax.grid.color  = gridColor();
      });
      c.update();
    });
  });
});

function syncThemeIcon(t) {
  document.querySelectorAll('.icon-sun').forEach(el => el.style.display = t === 'dark' ? '' : 'none');
  document.querySelectorAll('.icon-moon').forEach(el => el.style.display = t === 'dark' ? 'none' : '');
}

// ── Language switcher ─────────────────────────────────────────────────────
document.querySelectorAll('.lang-tab').forEach(btn => {
  btn.addEventListener('click', () => applyLang(btn.dataset.lang));
});

// Apply stored language on load (after DOM ready, handled below)

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
      showGateErr(t('gate.err.verify'));
      $('enter-btn').disabled = false;
    }
  } catch {
    showGateErr(t('gate.err.network'));
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

// ── Page navigation ───────────────────────────────────────────────────────
let wordlistsLoaded = false;
let lastData   = null;
let lastWlData = null;

document.querySelectorAll('.page-tab').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.page-tab').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    const page = btn.dataset.page;
    $('page-dashboard').hidden = page !== 'dashboard';
    $('page-wordlists').hidden = page !== 'wordlists';
    $('dashboard-controls').style.display = page === 'dashboard' ? 'flex' : 'none';
    if (page === 'wordlists' && !wordlistsLoaded) loadWordlists();
  });
});

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

// ── Dashboard fetch ───────────────────────────────────────────────────────
async function loadStats() {
  $('loading').hidden    = false;
  $('stats-root').hidden = true;
  $('loading').innerHTML = `<div class="spinner"></div><p>${t('loading.stats.spin')}</p>`;

  for (;;) {
    try {
      const res  = await fetch(`/api/stats?window=${currentWindow}`);
      if (res.status === 503) {
        await new Promise(r => setTimeout(r, 3000));
        continue;
      }
      if (!res.ok) throw new Error(`server error ${res.status}`);
      const data = await res.json();
      if (data.error) throw new Error(data.error);
      lastData = data;
      destroyCharts();
      renderAll(data);
      $('loading').hidden    = true;
      $('stats-root').hidden = false;
      $('last-updated').textContent = new Date().toLocaleTimeString();
      return;
    } catch (err) {
      $('loading').innerHTML =
        `<p style="color:var(--c-red);font-family:'JetBrains Mono',monospace;text-align:center">
          ERR: ${esc(err.message)}
        </p>`;
      return;
    }
  }
}

function destroyCharts() {
  Object.values(activeCharts).forEach(c => c.destroy());
  activeCharts = {};
}

// ── Dashboard render ──────────────────────────────────────────────────────
function renderAll(d) {
  const { overview } = d;

  counter($('v-auth'),             overview.auth_attempts);
  counter($('v-commands'),         overview.commands);
  counter($('v-ips'),              overview.unique_ips);
  counter($('v-downloads'),        overview.downloads);
  if (overview.unique_passwords     != null) counter($('v-unique-passwords'),  overview.unique_passwords);
  if (overview.unique_usernames     != null) counter($('v-unique-usernames'),  overview.unique_usernames);
  if (overview.unique_malware_hashes != null) counter($('v-malware-hashes'), overview.unique_malware_hashes);

  $('m-auth').textContent     = tf('meta.success_pct', overview.success_pct);
  $('m-commands').textContent = tf('meta.cmd_sessions', overview.cmd_sessions);

  if (d.timeseries.length) {
    const labels  = d.timeseries.map(r => fmtTime(r.t, currentWindow));
    activeCharts['ts'] = new Chart($('chart-timeseries'), {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: t('legend.failed'),
            data: d.timeseries.map(r => r.failed),
            borderColor: GBX.red, backgroundColor: GBX.red + '33',
            fill: true, tension: .35, pointRadius: 0, borderWidth: 2,
          },
          {
            label: t('legend.successful'),
            data: d.timeseries.map(r => r.successful),
            borderColor: GBX.green, backgroundColor: GBX.green + '33',
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

  {
    const counts = new Array(24).fill(0);
    d.by_hour.forEach(r => { counts[r.h] = Number(r.attempts); });
    const labels = counts.map((_, i) => `${String(i).padStart(2,'0')}:00`);
    activeCharts['hour'] = barChart('chart-hour', labels, counts, GBX.blue + 'cc');
  }

  {
    const dow = t('dow');
    const counts = new Array(7).fill(0);
    d.by_dow.forEach(r => { counts[r.dow] = Number(r.attempts); });
    activeCharts['dow'] = barChart('chart-dow', dow, counts, GBX.orange + 'cc');
  }

  if (d.top_usernames.length) {
    activeCharts['users'] = hbarChart('chart-usernames',
      d.top_usernames.map(r => r.username),
      d.top_usernames.map(r => Number(r.attempts)),
      GBX.aqua + 'cc');
  }

  if (d.top_passwords.length) {
    activeCharts['pw'] = hbarChart('chart-passwords',
      d.top_passwords.map(r => r.password),
      d.top_passwords.map(r => Number(r.attempts)),
      GBX.purple + 'cc');
  }

  if (d.top_urls.length) {
    activeCharts['urls'] = hbarChart('chart-urls',
      d.top_urls.map(r => r.url.replace(/^https?:\/\//, '').substring(0, 40)),
      d.top_urls.map(r => Number(r.downloads)),
      GBX.yellow + 'cc');
  }

  fillTable('tbl-password-hashes', d.password_hashes || [], r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono">${esc(r.password)}</td>
     <td class="mono truncate" title="${esc(r.sha256)}">${esc(r.sha256)}</td>
     <td class="num mono">${r.attempts}</td>`
  );

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
     <td class="mono">${r.shasum ? `<span title="${esc(r.shasum)}">${esc(r.shasum.substring(0,14))}…</span>` : '<span style="color:var(--fg-dim)">—</span>'}</td>`
  );

  $('section-downloads').hidden = !d.dl_log.length && !d.top_urls.length;

  const mhd = d.malware_hashes_detail || [];
  fillTable('tbl-malware-hashes', mhd, r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono truncate" title="${esc(r.shasum)}">${esc(r.shasum.substring(0, 16))}…</td>
     <td class="mono truncate">${esc(r.url || '—')}</td>
     <td class="num mono">${r.downloads}</td>
     <td class="mono">${fmtTs(r.first_seen)}</td>`
  );
}

// ── Wordlists ─────────────────────────────────────────────────────────────
let currentPeriod = 'daily';

document.querySelectorAll('[data-period]').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('[data-period]').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    currentPeriod = btn.dataset.period;
    loadWordlists();
  });
});

async function loadWordlists() {
  $('wl-loading').hidden = false;
  $('wl-root').hidden    = true;
  $('wl-loading').innerHTML = `<div class="spinner"></div><p>${t('loading.wordlists')}</p>`;

  try {
    const res  = await fetch(`/api/wordlist?period=${currentPeriod}`);
    if (!res.ok) throw new Error(`server error ${res.status}`);
    const data = await res.json();
    if (data.error) throw new Error(data.error);
    lastWlData      = data;
    wordlistsLoaded = true;
    renderWordlists(data);
    $('wl-loading').hidden = true;
    $('wl-root').hidden    = false;
  } catch (err) {
    $('wl-loading').innerHTML =
      `<p style="color:var(--c-red);font-family:'JetBrains Mono',monospace;text-align:center;line-height:1.8">
        ERR: ${esc(err.message)}
      </p>`;
  }
}

function renderWordlists(data) {
  for (const [wtype, info] of Object.entries(data)) {
    const countEl  = $(`wl-count-${wtype}`);
    const periodEl = $(`wl-period-${wtype}`);
    const sizeEl   = $(`wl-size-${wtype}`);
    const listEl   = $(`wl-preview-${wtype}`);
    if (!countEl) continue;

    countEl.textContent  = info.total ? info.total.toLocaleString() : '0';
    periodEl.textContent = info.oldest ? fmtPeriod(info.oldest, info.newest) : (info.ready ? t(`wl.period.${data.period}`) : t('period.nodata'));
    sizeEl.textContent   = info.gz_size || '—';

    listEl.innerHTML = (!info.preview || !info.preview.length)
      ? `<div class="wl-empty">${t('wl.empty')}</div>`
      : info.preview.map(v => `<div class="wl-preview-entry">${esc(v)}</div>`).join('');
  }
}

document.addEventListener('click', e => {
  const btn = e.target.closest('.btn-download');
  if (!btn) return;
  const wtype = btn.dataset.wtype;
  if (!wtype) return;
  btn.disabled = true;
  const a = document.createElement('a');
  a.href = `/api/wordlist/${wtype}/download?period=${currentPeriod}`;
  a.download = `autopot_${currentPeriod}_${wtype}.txt.gz`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => { btn.disabled = false; }, 2000);
});

// ── Counter animation ─────────────────────────────────────────────────────
function counter(el, target) {
  if (!el) return;
  if (!Number.isFinite(target) || target === 0) { el.textContent = target; return; }
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
    tbody.innerHTML = `<tr><td colspan="10" style="text-align:center;color:var(--fg-dim);padding:20px">${t('tbl.nodata')}</td></tr>`;
    return;
  }
  tbody.innerHTML = rows.map((r, i) => `<tr>${rowHtml({ ...r, _rank: i + 1 })}</tr>`).join('');
}

// ── Init ──────────────────────────────────────────────────────────────────
// Apply stored language preference on page load
if (currentLang !== 'en') applyLang(currentLang);
