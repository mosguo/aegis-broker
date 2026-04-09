# frontend-python

前端控制台目前採用單頁靜態介面，由 `server.py` 提供 `index.html` 與執行時 `config.js`。

## 目前功能

- Zeabur 風格左側欄位與底部用戶卡片
- 左下三點選單整合用戶屬性操作
- `GET /health/live` 與 `GET /health/ready` 自動輪詢
- 若有 Session Token，會自動查詢 `GET /v1/me/profile`
- `PUT /v1/me/profile` 可更新 `display_name`、`avatar_url`、`locale`、`reason_code`
- `GET /auth/google/start` 可啟動 Google OAuth 流程

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
