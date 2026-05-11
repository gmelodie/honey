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
    'card.novel_passwords':       'Novel Passwords',
    'card.novel_passwords.meta':  'not in common lists',

    'section.campaigns':    'Active Campaign Alerts',
    'section.geo':          'Attack Origins',
    'section.new_asns':     'New ASNs This Window',
    'chart.countries':      'Top Countries',
    'chart.asns':           'Top ASNs',
    'th.asn_org':           'Organization',
    'campaign.pattern.novel':       'Novel credentials — possible new tool',
    'campaign.pattern.established': 'Established credentials — known botnet',

    'section.overview':        'Overview',
    'section.timeseries':      'Login Attempts Over Time',
    'section.timing':          'Timing Patterns',
    'section.credentials':     'Top Credentials',
    'section.pairs':           'Top Credential Pairs',
    'section.clients':         'SSH Client Versions',
    'section.downloads':       'Downloads',
    'section.malware_hashes':  'Malware Hashes',
    'section.web':             'HTTP Honeypot',
    'section.web_timeseries':  'HTTP Traffic Over Time',
    'section.web_creds':       'Form Submission Credentials',
    'section.web_uas':         'HTTP User Agents',
    'section.logs':            'Activity Logs',

    'card.web_visits':          'HTTP Visits',
    'card.web_visits.meta':     'total HTTP requests logged',
    'card.web_ips':             'Unique IPs',
    'card.web_ips.meta':        'distinct web attacker IPs',
    'card.web_submissions':     'Login Attempts',
    'card.web_submissions.meta':'form submissions captured',
    'card.web_paths':           'Unique Paths',
    'card.web_paths.meta':      'distinct URLs probed',
    'card.web_uas':             'Unique User Agents',
    'card.web_uas.meta':        'distinct HTTP clients seen',

    'chart.web_paths':      'Top Requested Paths',
    'chart.web_ips':        'Top Source IPs',
    'chart.web_usernames':  'Usernames',
    'chart.web_passwords':  'Passwords',

    'legend.visits':      'Visits',
    'legend.submissions': 'Login Attempts',

    'th.user_agent': 'User Agent',
    'th.visits':     'Visits',
    'th.method':     'Method',
    'th.path':       'Path',

    'tab.web_visits':      'HTTP Visits',
    'tab.web_submissions': 'Form Submissions',

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
    'wl.usernames':        'Usernames',
    'wl.passwords':        'Passwords',
    'wl.pairs':            'Credential Pairs',
    'wl.novel_passwords':  'Novel Passwords',
    'wl.trending_passwords': 'Trending Passwords',
    'wl.dying_passwords':    'Dying Passwords',
    'wl.entries':       'entries',
    'wl.period':        'period',
    'wl.size':          '~size (gz)',
    'wl.preview':                'preview — top 5 by frequency',
    'wl.preview.pairs':          'preview — top 5 by frequency (user:pass format)',
    'wl.preview.trending_passwords': 'preview — top 5 fastest growing',
    'wl.preview.dying_passwords':    'preview — top 5 fastest declining',
    'wl.preview.novel_passwords': 'preview — top 5 not in common lists',
    'wl.trending_passwords.tip': 'Passwords growing in attack frequency compared to the previous period — sorted by relative growth rate.',
    'wl.dying_passwords.tip':    'Passwords declining in attack frequency compared to the previous period — sorted by absolute drop in usage.',
    'wl.novel_passwords.tip': 'Passwords not found in common reference wordlists — filtered against xato-net 1 M, NCSC 100 k, Pwdb top 1 M, probable top 12 k, and darkweb 2017 top 10 k.',
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
    'card.novel_passwords':       'Senhas Novas',
    'card.novel_passwords.meta':  'não estão em listas comuns',

    'section.campaigns':    'Alertas de Campanha Ativa',
    'section.geo':          'Origens dos Ataques',
    'section.new_asns':     'Novos ASNs Neste Período',
    'chart.countries':      'Top Países',
    'chart.asns':           'Top ASNs',
    'th.asn_org':           'Organização',
    'campaign.pattern.novel':       'Credenciais novas — possível nova ferramenta',
    'campaign.pattern.established': 'Credenciais conhecidas — botnet estabelecido',

    'section.overview':        'Visão Geral',
    'section.timeseries':      'Tentativas de Login ao Longo do Tempo',
    'section.timing':          'Padrões Temporais',
    'section.credentials':     'Credenciais Mais Usadas',
    'section.pairs':           'Pares de Credenciais Mais Usados',
    'section.clients':         'Versões de Cliente SSH',
    'section.downloads':       'Downloads',
    'section.malware_hashes':  'Hashes de Malware',
    'section.web':             'Honeypot HTTP',
    'section.web_timeseries':  'Tráfego HTTP ao Longo do Tempo',
    'section.web_creds':       'Credenciais de Formulários',
    'section.web_uas':         'User Agents HTTP',
    'section.logs':            'Registros de Atividade',

    'card.web_visits':          'Visitas HTTP',
    'card.web_visits.meta':     'requisições HTTP registradas',
    'card.web_ips':             'IPs Únicos',
    'card.web_ips.meta':        'IPs de atacantes web distintos',
    'card.web_submissions':     'Tentativas de Login',
    'card.web_submissions.meta':'formulários capturados',
    'card.web_paths':           'Caminhos Únicos',
    'card.web_paths.meta':      'URLs sondadas distintas',
    'card.web_uas':             'User Agents Únicos',
    'card.web_uas.meta':        'clientes HTTP distintos',

    'chart.web_paths':      'Caminhos Mais Requisitados',
    'chart.web_ips':        'IPs de Origem',
    'chart.web_usernames':  'Usuários',
    'chart.web_passwords':  'Senhas',

    'legend.visits':      'Visitas',
    'legend.submissions': 'Tentativas de Login',

    'th.user_agent': 'User Agent',
    'th.visits':     'Visitas',
    'th.method':     'Método',
    'th.path':       'Caminho',

    'tab.web_visits':      'Visitas HTTP',
    'tab.web_submissions': 'Envios de Formulário',

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
    'wl.usernames':        'Usuários',
    'wl.passwords':        'Senhas',
    'wl.pairs':            'Pares de Credenciais',
    'wl.novel_passwords':  'Senhas Novas',
    'wl.trending_passwords': 'Senhas em Alta',
    'wl.dying_passwords':    'Senhas em Queda',
    'wl.entries':       'entradas',
    'wl.period':        'período',
    'wl.size':          '~tamanho (gz)',
    'wl.preview':                'prévia — top 20 por frequência',
    'wl.preview.pairs':          'prévia — top 20 por frequência (formato usuário:senha)',
    'wl.preview.trending_passwords': 'prévia — top 5 crescendo mais rápido',
    'wl.preview.dying_passwords':    'prévia — top 5 declinando mais rápido',
    'wl.preview.novel_passwords': 'prévia — top 5 não estão em listas comuns',
    'wl.trending_passwords.tip': 'Senhas com frequência de ataque crescente em comparação ao período anterior — ordenadas pela taxa de crescimento relativo.',
    'wl.dying_passwords.tip':    'Senhas com frequência de ataque declinante em comparação ao período anterior — ordenadas pela queda absoluta no uso.',
    'wl.novel_passwords.tip': 'Senhas não encontradas em listas de referência comuns — filtradas contra xato-net 1 M, NCSC 100 k, Pwdb top 1 M, probable top 12 k e darkweb 2017 top 10 k.',
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

  document.querySelectorAll('[data-i18n-tip]').forEach(el => {
    const val = t(el.dataset.i18nTip);
    if (typeof val === 'string') el.dataset.tip = val;
  });

  document.querySelectorAll('.lang-tab').forEach(b => {
    b.classList.toggle('active', b.dataset.lang === lang);
  });

  // Re-render live data with new language (translated DOW labels, empty states, etc.)
  if (lastData && (!$('page-ssh').hidden || !$('page-http').hidden)) {
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
    $('page-ssh').hidden       = page !== 'ssh';
    $('page-http').hidden      = page !== 'http';
    $('page-wordlists').hidden = page !== 'wordlists';
    $('dashboard-controls').style.display = page === 'wordlists' ? 'none' : 'flex';
    if (page === 'wordlists' && !wordlistsLoaded) loadWordlists();
    if (page === 'http') requestAnimationFrame(() => Object.values(activeCharts).forEach(c => c.resize()));
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
document.querySelectorAll('.log-tab[data-tab]').forEach(btn => {
  btn.addEventListener('click', () => {
    const section = btn.closest('section');
    section.querySelectorAll('.log-tab').forEach(b => b.classList.remove('active'));
    section.querySelectorAll('.log-panel').forEach(p => p.hidden = true);
    btn.classList.add('active');
    $('tab-' + btn.dataset.tab).hidden = false;
  });
});

// ── Stats fetch ───────────────────────────────────────────────────────────
async function loadStats() {
  $('loading').hidden    = false;
  $('stats-root').hidden = true;
  $('http-loading').hidden = false;
  $('http-root').hidden    = true;
  $('loading').innerHTML = `<div class="spinner"></div><p>${t('loading.stats.spin')}</p>`;
  $('http-loading').innerHTML = `<div class="spinner"></div><p>${t('loading.stats.spin')}</p>`;

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
      $('loading').hidden      = true;
      $('stats-root').hidden   = false;
      $('http-loading').hidden = true;
      $('http-root').hidden    = false;
      $('last-updated').textContent = new Date().toLocaleTimeString();
      return;
    } catch (err) {
      const msg = `<p style="color:var(--c-red);font-family:'JetBrains Mono',monospace;text-align:center">ERR: ${esc(err.message)}</p>`;
      $('loading').innerHTML     = msg;
      $('http-loading').innerHTML = msg;
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
  if (overview.unique_malware_hashes != null) counter($('v-malware-hashes'),    overview.unique_malware_hashes);
  if (overview.novel_passwords       != null) counter($('v-novel-passwords'),   overview.novel_passwords);

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

  // ── Web honeypot ──────────────────────────────────────────────────────────
  const web = d.web || {};
  const wo  = web.overview || {};

  if (wo.visits            != null) counter($('v-web-visits'),            wo.visits);
  if (wo.unique_ips        != null) counter($('v-web-ips'),               wo.unique_ips);
  if (wo.submissions       != null) counter($('v-web-submissions'),       wo.submissions);
  if (wo.unique_paths      != null) counter($('v-web-paths'),             wo.unique_paths);
  if (wo.unique_uas        != null) counter($('v-web-uas'),               wo.unique_uas);
  if (wo.unique_passwords  != null) counter($('v-web-unique-passwords'),  wo.unique_passwords);
  if (wo.unique_usernames  != null) counter($('v-web-unique-usernames'),  wo.unique_usernames);

  if (web.timeseries && web.timeseries.length) {
    const labels = web.timeseries.map(r => fmtTime(r.t, currentWindow));
    activeCharts['web-ts'] = new Chart($('chart-web-timeseries'), {
      type: 'line',
      data: {
        labels,
        datasets: [
          {
            label: t('legend.visits'),
            data: web.timeseries.map(r => r.visits),
            borderColor: GBX.blue, backgroundColor: GBX.blue + '33',
            fill: true, tension: .35, pointRadius: 0, borderWidth: 2,
          },
          {
            label: t('legend.submissions'),
            data: web.timeseries.map(r => r.submissions),
            borderColor: GBX.red, backgroundColor: GBX.red + '33',
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

  if (web.top_paths && web.top_paths.length) {
    activeCharts['web-paths'] = hbarChart('chart-web-paths',
      web.top_paths.map(r => r.path),
      web.top_paths.map(r => Number(r.visits)),
      GBX.blue + 'cc');
  }

  if (web.top_ips && web.top_ips.length) {
    activeCharts['web-ips'] = hbarChart('chart-web-ips',
      web.top_ips.map(r => r.ip),
      web.top_ips.map(r => Number(r.visits)),
      GBX.orange + 'cc');
  }

  const webHasCreds = (web.top_usernames && web.top_usernames.length) ||
                      (web.top_passwords && web.top_passwords.length);
  $('section-web-creds').hidden = !webHasCreds;

  if (web.top_usernames && web.top_usernames.length) {
    activeCharts['web-users'] = hbarChart('chart-web-usernames',
      web.top_usernames.map(r => r.username),
      web.top_usernames.map(r => Number(r.attempts)),
      GBX.aqua + 'cc');
  }

  if (web.top_passwords && web.top_passwords.length) {
    activeCharts['web-pw'] = hbarChart('chart-web-passwords',
      web.top_passwords.map(r => r.password),
      web.top_passwords.map(r => Number(r.attempts)),
      GBX.purple + 'cc');
  }

  fillTable('tbl-web-uas', web.top_uas || [], r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono truncate">${esc(r.user_agent)}</td>
     <td class="num mono">${r.visits}</td>`
  );

  fillTable('tbl-web-visits', web.visit_log || [], r =>
    `<td class="mono">${fmtTs(r.time)}</td>
     <td class="mono">${esc(r.ip)}</td>
     <td class="mono">${esc(r.method)}</td>
     <td class="mono truncate">${esc(r.path)}</td>
     <td class="mono truncate">${esc(r.user_agent)}</td>`
  );

  fillTable('tbl-web-submissions', web.submission_log || [], r => {
    const fd = r.form_data || {};
    return `<td class="mono">${fmtTs(r.time)}</td>
     <td class="mono">${esc(r.ip)}</td>
     <td class="mono">${esc(fd.username || '—')}</td>
     <td class="mono">${esc(fd.password || '—')}</td>`;
  });

  // ── Geo / ASN ─────────────────────────────────────────────────────────────
  const geo = d.geo || {};
  const hasGeo = (geo.top_countries && geo.top_countries.length) ||
                 (geo.top_asns && geo.top_asns.length);
  $('section-geo').hidden = !hasGeo;

  if (geo.top_countries && geo.top_countries.length) {
    choroplethChart('chart-countries', geo.top_countries).then(chart => {
      activeCharts['geo-countries'] = chart;
    });
  }

  if (geo.top_asns && geo.top_asns.length) {
    activeCharts['geo-asns'] = hbarChart('chart-asns',
      geo.top_asns.map(r => r.asn_org ? `AS${r.asn} ${r.asn_org}` : `AS${r.asn}`),
      geo.top_asns.map(r => r.sessions),
      GBX.orange + 'cc');
  }

  const newAsns = geo.new_asns || [];
  $('section-new-asns').hidden = !newAsns.length;
  fillTable('tbl-new-asns', newAsns, r =>
    `<td class="rank">${r._rank}</td>
     <td class="mono">AS${r.asn}</td>
     <td class="mono truncate">${esc(r.asn_org || '—')}</td>
     <td class="mono">${fmtTs(r.first_seen)}</td>
     <td class="num mono">${Number(r.attempts).toLocaleString()}</td>`
  );

  // ── Campaign alerts ───────────────────────────────────────────────────────
  const campaigns = d.campaigns || [];
  $('section-campaigns').hidden = !campaigns.length;

  if (campaigns.length) {
    $('campaign-alerts-list').innerHTML = campaigns.map(c => {
      const isNovel      = c.credential_pattern === 'novel';
      const patternLabel = t(isNovel ? 'campaign.pattern.novel' : 'campaign.pattern.established');
      const asnRows = (c.new_asns || []).map(a =>
        `<tr>
          <td class="mono">AS${a.asn}</td>
          <td class="mono truncate">${esc(a.asn_org || '—')}</td>
          <td class="num mono">${Number(a.attempts).toLocaleString()}</td>
        </tr>`
      ).join('');
      return `
        <div class="campaign-card">
          <div class="campaign-header">
            <span class="campaign-label">CAMPAIGN #${c.id}</span>
            <span class="campaign-badge ${isNovel ? 'badge-campaign-novel' : 'badge-campaign-established'}">${esc(patternLabel)}</span>
          </div>
          <div class="campaign-stats">
            <div class="campaign-stat">
              <span class="campaign-stat-label">Onset</span>
              <span class="campaign-stat-val mono">${fmtTs(c.onset_time)}</span>
            </div>
            <div class="campaign-stat">
              <span class="campaign-stat-label">Z-Score</span>
              <span class="campaign-stat-val mono" style="color:var(--c-red)">${c.z_score.toFixed(2)}</span>
            </div>
            <div class="campaign-stat">
              <span class="campaign-stat-label">Peak/hr</span>
              <span class="campaign-stat-val mono">${Math.round(c.peak_rate_per_hour).toLocaleString()}</span>
            </div>
            <div class="campaign-stat">
              <span class="campaign-stat-label">Baseline/hr</span>
              <span class="campaign-stat-val mono">${Math.round(c.baseline_rate_per_hour).toLocaleString()}</span>
            </div>
            <div class="campaign-stat">
              <span class="campaign-stat-label">New ASNs</span>
              <span class="campaign-stat-val mono">${c.new_asn_count}</span>
            </div>
            <div class="campaign-stat">
              <span class="campaign-stat-label">New-ASN Traffic</span>
              <span class="campaign-stat-val mono">${Math.round(c.spike_ratio * 100)}%</span>
            </div>
          </div>
          ${asnRows ? `<table class="data-tbl" style="margin-top:10px">
            <thead><tr><th>ASN</th><th data-i18n="th.asn_org">Organization</th><th data-i18n="th.attempts">Attempts</th></tr></thead>
            <tbody>${asnRows}</tbody>
          </table>` : ''}
        </div>`;
    }).join('');
  }
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

// ── Choropleth world map ──────────────────────────────────────────────────
let _worldAtlas = null;
async function _fetchWorldAtlas() {
  if (!_worldAtlas) {
    const res = await fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json');
    _worldAtlas = await res.json();
  }
  return _worldAtlas;
}

// ISO 3166-1 alpha-2 → numeric
const ISO2_NUM = {
  AF:4,AX:248,AL:8,DZ:12,AS:16,AD:20,AO:24,AI:660,AG:28,AR:32,AM:51,AW:533,
  AU:36,AT:40,AZ:31,BS:44,BH:48,BD:50,BB:52,BY:112,BE:56,BZ:84,BJ:204,BM:60,
  BT:64,BO:68,BQ:535,BA:70,BW:72,BR:76,IO:86,VG:92,BN:96,BG:100,BF:854,BI:108,
  CV:132,KH:116,CM:120,CA:124,KY:136,CF:140,TD:148,CL:152,CN:156,HK:344,MO:446,
  CX:162,CC:166,CO:170,KM:174,CG:178,CD:180,CK:184,CR:188,HR:191,CU:192,CW:531,
  CY:196,CZ:203,CI:384,DK:208,DJ:262,DM:212,DO:214,EC:218,EG:818,SV:222,GQ:226,
  ER:232,EE:233,SZ:748,ET:231,FK:238,FO:234,FJ:242,FI:246,FR:250,GF:254,PF:258,
  TF:260,GA:266,GM:270,GE:268,DE:276,GH:288,GI:292,GR:300,GL:304,GD:308,GP:312,
  GU:316,GT:320,GG:831,GN:324,GW:624,GY:328,HT:332,HM:334,VA:336,HN:340,HU:348,
  IS:352,IN:356,ID:360,IR:364,IQ:368,IE:372,IM:833,IL:376,IT:380,JM:388,JP:392,
  JE:832,JO:400,KZ:398,KE:404,KI:296,KP:408,KR:410,KW:414,KG:417,LA:418,LV:428,
  LB:422,LS:426,LR:430,LY:434,LI:438,LT:440,LU:442,MG:450,MW:454,MY:458,MV:462,
  ML:466,MT:470,MH:584,MQ:474,MR:478,MU:480,YT:175,MX:484,FM:583,MD:498,MC:492,
  MN:496,ME:499,MS:500,MA:504,MZ:508,MM:104,NA:516,NR:520,NP:524,NL:528,NC:540,
  NZ:554,NI:558,NE:562,NG:566,NU:570,NF:574,MK:807,MP:580,NO:578,OM:512,PK:586,
  PW:585,PS:275,PA:591,PG:598,PY:600,PE:604,PH:608,PN:612,PL:616,PT:620,PR:630,
  QA:634,RE:638,RO:642,RU:643,RW:646,BL:652,SH:654,KN:659,LC:662,MF:663,PM:666,
  VC:670,WS:882,SM:674,ST:678,SA:682,SN:686,RS:688,SC:690,SL:694,SG:702,SX:534,
  SK:703,SI:705,SB:90,SO:706,ZA:710,GS:239,SS:728,ES:724,LK:144,SD:729,SR:740,
  SJ:744,SE:752,CH:756,SY:760,TW:158,TJ:762,TZ:834,TH:764,TL:626,TG:768,TK:772,
  TO:776,TT:780,TN:788,TR:792,TM:795,TC:796,TV:798,UM:581,VI:850,UG:800,UA:804,
  AE:784,GB:826,US:840,UY:858,UZ:860,VU:548,VE:862,VN:704,WF:876,EH:732,YE:887,
  ZM:894,ZW:716,XK:383,KP:408,SS:728,
};

async function choroplethChart(canvasId, countryData) {
  const existing = Chart.getChart(canvasId);
  if (existing) existing.destroy();

  const topology = await _fetchWorldAtlas();
  const features = topojson.feature(topology, topology.objects.countries).features;

  const byNum = {};
  for (const r of countryData) {
    const n = ISO2_NUM[r.country_iso];
    if (n) byNum[n] = r.sessions;
  }
  const maxVal = Math.max(...Object.values(byNum), 1);
  const dark = document.documentElement.dataset.theme === 'dark';

  return new Chart($(canvasId), {
    type: 'choropleth',
    data: {
      labels: features.map(f => f.properties.name),
      datasets: [{
        data: features.map(f => ({ feature: f, value: byNum[+f.id] ?? 0 })),
        backgroundColor(ctx) {
          const v = ctx.dataset.data[ctx.dataIndex]?.value ?? 0;
          if (!v) return dark ? '#1e2025' : '#e8eaed';
          const a = (0.18 + 0.82 * (v / maxVal)).toFixed(2);
          return `rgba(220,50,47,${a})`;
        },
        borderColor: dark ? '#2e3138' : '#c8ccd2',
        borderWidth: 0.4,
      }],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
        tooltip: {
          callbacks: {
            label(ctx) {
              const v = ctx.dataset.data[ctx.dataIndex]?.value;
              return v ? ` ${v.toLocaleString()} sessions` : ' no data';
            },
          },
        },
      },
      scales: {
        projection: { type: 'projection', axis: 'x', projection: 'equalEarth', position: 'chartArea', display: false },
        color:      { type: 'color',      axis: 'x', display: false },
      },
    },
  });
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
