# AegisBroker Development Plan

> Last Updated: 2026-04-09  
> Version: v0.1.3-plan

## 本次前端調整

本輪更新聚焦在控制台與 Zeabur 參考畫面的對齊，目標是讓前端展示層更接近部署環境的操作習慣，同時不改變後端事件、審計與狀態機架構。

### 1. 左下用戶操作區重整

- 改為帳號卡片 + 三點選單形式
- 改為浮動置下，不再跟隨頁面內容沉到底部
- 姓名字級統一為 `14px`
- 角色與附屬資訊字級統一為 `12px`
- 選單內容改為：
  - 載入個人資料
  - 複製 Workspace ID
  - 複製 Session Token
  - 切換登入狀態

### 2. 系統狀態三組自動查詢

控制台右側狀態區改為三組固定輪詢：

1. `Backend API`
   - 查詢 `GET /health/live`
2. `資料庫 Ready`
   - 查詢 `GET /health/ready`
3. `使用者 Session`
   - 當存在 Session Token 時，查詢 `GET /v1/me/profile`

### 3. 輪詢策略

- 頁面載入後立即自動查詢一次
- 每 `30` 秒固定再查詢一次
- 保留手動刷新按鈕，供 Zeabur 部署後即時確認

### 4. Zeabur 單網域代理策略

為避免 Zeabur 前端網域直接請求 `/health/live`、`/health/ready`、`/auth/google/start`、`/v1/me/profile` 時回傳靜態 404，前端服務補上代理層：

- `/health/*` 代理到 backend
- `/auth/google/*` 代理到 backend
- `/v1/*` 代理到 backend

這讓同一個公開網域可以同時承接：

- 控制台畫面
- 健康檢查
- OAuth 啟動入口
- profile/session API

### 5. 比較圖說

控制台首頁新增「介面比較圖說」區塊，以文字方式對照：

- Zeabur 參考畫面
- AegisBroker 目前執行畫面

此區塊用來保存設計對齊意圖，方便後續繼續往 Zeabur 的資訊階層與互動節奏收斂。

## 與架構文件的關係

這次變更僅調整前端操作體驗與狀態呈現，不改動：

- 事件寫入規則
- 審計鏈要求
- 狀態機驗證
- workspace server-side scope 原則

前端仍然只透過既有 API：

- `/health/live`
- `/health/ready`
- `/auth/google/start`
- `/v1/me/profile`

來顯示控制面板狀態與用戶資料。
