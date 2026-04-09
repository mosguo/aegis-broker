# frontend-python

AegisBroker 前端（v0.1.1）提供可直接部署的靜態控制台頁面。

## 功能

- i18n 語系切換：`zh-TW`、`zh-CN`、`en`、`es`、`tr`
- 後端健康檢查觸發（`/health/live`、`/health/ready`）
- Google OAuth 啟動（`/auth/google/start`）
- Session Token 輸入與 `GET /v1/me/profile` 讀取
- `PUT /v1/me/profile` 個人資料更新（display_name / avatar_url / locale / reason_code）

## Local run

```bash
cd frontend-python
python server.py
```


## Runtime config

`server.py` 會提供 `/config.js`，將以下環境變數注入前端：

- `BACKEND_API_BASE_URL`：後端 API 基底 URL（建議在 Zeabur 設定）
- `DEFAULT_LOCALE`：預設語系

若部署後前端與後端是不同網域，也可在頁面上的 **Backend API Base URL** 欄位直接覆寫，會儲存在瀏覽器 localStorage（key: `aegis.backendBaseUrl`）。
