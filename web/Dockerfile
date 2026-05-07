FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
CMD ["gunicorn", "--bind", "127.0.0.1:8373", "--workers", "2", "--timeout", "30", "app:app"]
