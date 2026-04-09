# frontend-python

前端控制台目前採用單頁靜態介面，由 `server.py` 提供 `index.html` 與執行時 `config.js`。

## 目前功能

- Zeabur 風格左側欄位與底部用戶卡片
- 左下三點選單整合用戶屬性操作，並以浮動置下方式固定在視窗下側
- `GET /health/live` 與 `GET /health/ready` 自動輪詢
- 若有 Session Token，會自動查詢 `GET /v1/me/profile`
- `PUT /v1/me/profile` 可更新 `display_name`、`avatar_url`、`locale`、`reason_code`
- `GET /auth/google/start` 可啟動 Google OAuth 流程
- 前端服務會代理以下路徑到 `BACKEND_API_BASE_URL`
  - `/health/*`
  - `/auth/google/*`
  - `/v1/*`

## 啟動方式

```bash
cd frontend-python
python server.py
```

## 執行時設定

`server.py` 會提供 `/config.js`，目前支援：

- `BACKEND_API_BASE_URL`
- `DEFAULT_LOCALE`

若頁面上修改 `Backend API Base URL`，會即時寫入 `localStorage` 的 `aegis.backendBaseUrl`。

## Zeabur 路由說明

若 Zeabur 對外入口是前端服務網域，例如：

- `https://mosguo.zeabur.app`

則以下網址現在可以直接由前端服務代理到後端：

- `https://mosguo.zeabur.app/health/live`
- `https://mosguo.zeabur.app/health/ready`
- `https://mosguo.zeabur.app/auth/google/start?workspace_id=<WORKSPACE_ID>`
- `https://mosguo.zeabur.app/v1/me/profile`
