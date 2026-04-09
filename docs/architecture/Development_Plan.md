# AegisBroker Development Plan

> Last Updated: 2026-04-09  
> Version: v0.1.1-plan

## 多語一致性原則補充

為符合 AegisBroker 控制型系統在全球化營運中的一致性要求，未來所有介面開發與系統訊息傳遞必須遵守下列規範：

1. **所有使用者介面（UI）必須支援五語切換**
   - `zh-TW`（繁體中文）
   - `zh-CN`（簡體中文）
   - `en`（English）
   - `es`（Español）
   - `tr`（Türkçe）

2. **所有系統非圖形介面對外訊息也必須支援五語切換**
   - 包含但不限於：
     - API 回應中的 user-facing message
     - 通知訊息（Email / Webhook / 站內訊息）
     - 匯出報表中的文字欄位
     - 背景作業回傳給使用者的狀態/錯誤說明

3. **錯誤碼與理由碼維持穩定，文案才做語系切換**
   - `error_code`、`reason_code` 必須維持 machine-usable 穩定值。
   - 前端與外部訊息層僅針對顯示文案做本地化（i18n）。

4. **開發與測試要求**
   - 新增功能時，必須同時提供五語文案鍵值。
   - 任何變更若新增使用者可見文案，PR 需附上 i18n key 清單。
   - 測試至少覆蓋：預設語系回退、缺漏 key 防護、五語可切換。

5. **文件與部署要求**
   - `infra/env/*` 應保留語系相關設定（如 `DEFAULT_LOCALE`, `ENABLED_LOCALES`）。
   - 架構文件與 API 合約若新增 user-facing message 欄位，需明確說明語系策略。

---

## Implementation Note

本文件為開發計劃補充條款，對後續所有功能迭代生效。若與舊文件衝突，依「五語一致性」原則優先，並在下一版架構文件中同步更新。
