"""
Fake RouterOS web admin — logs every interaction, traps credential harvesters.
"""
import json
import logging
import os
from datetime import datetime, timezone
from flask import Flask, request, make_response

app = Flask(__name__)

LOG_FILE = os.environ.get("HONEYPOT_LOG", "/var/log/honeypot-web.log")
os.makedirs(os.path.dirname(LOG_FILE), exist_ok=True)

logging.basicConfig(
    filename=LOG_FILE,
    level=logging.INFO,
    format="%(message)s",
)

_LOGIN_PAGE = """\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>RouterOS - Login</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:Arial,sans-serif;background:#1e1e1e;color:#ccc;display:flex;
  align-items:center;justify-content:center;min-height:100vh}
.box{width:340px;background:#2a2a2a;border:1px solid #444;border-radius:4px;padding:32px}
h1{font-size:18px;color:#fff;margin-bottom:4px}
.sub{font-size:12px;color:#888;margin-bottom:24px}
label{display:block;font-size:12px;margin-bottom:4px;color:#aaa}
input{width:100%;padding:8px 10px;background:#1a1a1a;border:1px solid #555;
  border-radius:3px;color:#eee;font-size:14px;margin-bottom:16px;outline:none}
input:focus{border-color:#0074d9}
button{width:100%;padding:10px;background:#0074d9;color:#fff;border:none;
  border-radius:3px;font-size:14px;cursor:pointer}
button:hover{background:#005fa3}
.err{color:#ff4c4c;font-size:12px;margin-top:12px;text-align:center}
</style>
</head>
<body>
<div class="box">
  <h1>MikroTik RouterOS</h1>
  <div class="sub">v7.15.3 (stable) &mdash; Please log in</div>
  <form method="POST" action="/login">
    <label>Username</label>
    <input type="text" name="username" autocomplete="off" placeholder="admin">
    <label>Password</label>
    <input type="password" name="password" autocomplete="off">
    <button type="submit">Log In</button>
    {err}
  </form>
</div>
</body>
</html>"""


def _log(extra=None):
    entry = {
        "ts":         datetime.now(timezone.utc).isoformat(),
        "ip":         request.headers.get("X-Real-IP") or request.remote_addr,
        "method":     request.method,
        "path":       request.full_path,
        "host":       request.host,
        "ua":         request.headers.get("User-Agent", ""),
    }
    if extra:
        entry.update(extra)
    logging.info(json.dumps(entry))


@app.route("/login", methods=["POST"])
def do_login():
    creds = {"username": request.form.get("username", ""), "password": request.form.get("password", "")}
    _log({"event": "credential_attempt", **creds})
    return make_response(_LOGIN_PAGE.format(err='<p class="err">Invalid username or password</p>'), 200)


@app.route("/", defaults={"path": ""}, methods=["GET", "POST"])
@app.route("/<path:path>", methods=["GET", "POST"])
def catch_all(path):
    _log({"event": "probe", "body": request.get_data(as_text=True)[:512] if request.method == "POST" else None})
    return make_response(_LOGIN_PAGE.format(err=""), 200)
