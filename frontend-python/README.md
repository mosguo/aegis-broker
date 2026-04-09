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
python -m http.server 3000
```
