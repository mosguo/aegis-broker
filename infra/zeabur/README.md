# Zeabur Deployment Config Pack (v0.1.0 target)

此目錄提供可直接在 Zeabur 建立三端服務的組態範本：

1. 前端 `frontend-python`
2. 後端 `backend-rust`
3. 資料庫 `postgres`

## Files

- `services/frontend-python.service.yaml`
- `services/backend-rust.service.yaml`
- `services/postgres.service.yaml`

## Usage

1. 在 Zeabur 建立對應 service。
2. 將相對應 YAML 欄位填入 Zeabur Service Settings（build/start/env/healthcheck）。
3. 先部署 postgres，再部署 backend，最後部署 frontend。
4. 確認 `backend /health/ready` 通過後再切正式流量。
