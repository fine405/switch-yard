import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import {
  loadPanelState,
  quitApp,
  setAutoSwitch,
  setUsageApi,
  switchAccount,
} from "./lib/tauri";
import type { AccountSummary, PanelState } from "./lib/types";
import "./App.css";

function App() {
  const mountedRef = useRef(true);
  const [panelState, setPanelState] = useState<PanelState | null>(null);
  const [loading, setLoading] = useState(true);
  const [busyAccountKey, setBusyAccountKey] = useState<string | null>(null);
  const [busyToggle, setBusyToggle] = useState<"auto" | "api" | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function refreshState(showSpinner = false) {
    if (showSpinner) {
      setLoading(true);
    }

    try {
      const nextState = await loadPanelState();
      if (!mountedRef.current) {
        return;
      }
      setPanelState(nextState);
      setError(null);
    } catch (nextError) {
      if (!mountedRef.current) {
        return;
      }
      setError(toErrorMessage(nextError));
    } finally {
      if (mountedRef.current && showSpinner) {
        setLoading(false);
      }
    }
  }

  async function handleSwitch(account: AccountSummary) {
    setBusyAccountKey(account.accountKey);
    try {
      const nextState = await switchAccount(account.accountKey);
      setPanelState(nextState);
      setError(null);
    } catch (nextError) {
      setError(toErrorMessage(nextError));
    } finally {
      setBusyAccountKey(null);
    }
  }

  async function handleAutoSwitch(enabled: boolean) {
    setBusyToggle("auto");
    try {
      const nextState = await setAutoSwitch(enabled);
      setPanelState(nextState);
      setError(null);
    } catch (nextError) {
      setError(toErrorMessage(nextError));
    } finally {
      setBusyToggle(null);
    }
  }

  async function handleUsageApi(enabled: boolean) {
    setBusyToggle("api");
    try {
      const nextState = await setUsageApi(enabled);
      setPanelState(nextState);
      setError(null);
    } catch (nextError) {
      setError(toErrorMessage(nextError));
    } finally {
      setBusyToggle(null);
    }
  }

  useEffect(() => {
    mountedRef.current = true;
    void refreshState(true);

    let unlisten: (() => void) | undefined;
    void listen("switchyard://registry-changed", () => {
      void refreshState(false);
    }).then((cleanup) => {
      if (mountedRef.current) {
        unlisten = cleanup;
      }
    });

    return () => {
      mountedRef.current = false;
      unlisten?.();
    };
  }, []);

  const accountCount = panelState?.accounts.length ?? 0;
  const statusLabel = panelState?.apiUsageEnabled ? "API 用量" : "本地用量";

  return (
    <main className="shell">
      <header className="shell__header">
        <div>
          <p className="eyebrow">macOS 菜单栏一期</p>
          <h1>Switchyard</h1>
          <p className="subtitle">
            {loading ? "正在读取账号状态..." : `已发现 ${accountCount} 个账号 · ${statusLabel}`}
          </p>
        </div>
        <div className="header-actions">
          <button className="button button--ghost" onClick={() => void refreshState(true)}>
            刷新
          </button>
          <span className="pill">{loading ? "同步中" : "就绪"}</span>
        </div>
      </header>

      {error ? (
        <section className="notice notice--error">
          <strong>操作失败</strong>
          <span>{error}</span>
        </section>
      ) : null}

      {!panelState?.hasRegistry ? (
        <section className="empty-state">
          <div className="empty-state__icon">◎</div>
          <h2>还没有可切换的账号</h2>
          <p>Switchyard 正在读取当前用户的 Codex 数据目录。</p>
          <code>{panelState?.registryPath ?? "~/.codex/accounts/registry.json"}</code>
          <p className="muted">
            先在终端完成 `codex login` 和账号导入，随后点击刷新即可。
          </p>
        </section>
      ) : (
        <section className="account-list">
          {panelState.accounts.map((account) => (
            <button
              key={account.accountKey}
              className={`account-card${account.isActive ? " account-card--active" : ""}`}
              disabled={busyAccountKey === account.accountKey}
              onClick={() => {
                if (!account.isActive) {
                  void handleSwitch(account);
                }
              }}
            >
              <div className="account-card__top">
                <div className="account-card__identity">
                  <span
                    className={`status-dot${account.isActive ? " status-dot--active" : ""}`}
                  />
                  <div>
                    <div className="account-card__title">{account.displayName}</div>
                    <div className="account-card__meta">
                      {account.hasAuthSnapshot ? "快照已就绪" : "缺少快照文件"}
                      {account.lastUsedAt ? ` · 最近使用 ${formatLastUsed(account.lastUsedAt)}` : ""}
                    </div>
                  </div>
                </div>
                <div className="account-card__badges">
                  {account.plan ? (
                    <span className={`plan-badge plan-badge--${planTone(account.plan)}`}>
                      {account.plan}
                    </span>
                  ) : null}
                  {account.isActive ? <span className="active-badge">当前</span> : null}
                </div>
              </div>

              <div className="usage-row">
                <UsageMeter label="5h" value={account.usage5hRemaining} />
                <UsageMeter label="周" value={account.usageWeeklyRemaining} />
              </div>
            </button>
          ))}
        </section>
      )}

      <section className="settings-panel">
        <SettingRow
          description="低额度时自动切换账号"
          disabled={!panelState || busyToggle !== null}
          label="Auto-switch"
          checked={panelState?.autoSwitchEnabled ?? false}
          onChange={(checked) => void handleAutoSwitch(checked)}
        />
        <SettingRow
          description="切换展示来源：API / 本地"
          disabled={!panelState || busyToggle !== null}
          label="Usage API"
          checked={panelState?.apiUsageEnabled ?? false}
          onChange={(checked) => void handleUsageApi(checked)}
        />
      </section>

      <footer className="shell__footer">
        <div>
          <div className="footer-label">数据目录</div>
          <code>{panelState?.codexHome ?? "~/.codex"}</code>
        </div>
        <button className="button button--danger" onClick={() => void quitApp()}>
          退出
        </button>
      </footer>
    </main>
  );
}

function UsageMeter({ label, value }: { label: string; value: number | null }) {
  if (value === null) {
    return (
      <div className="usage-meter usage-meter--empty">
        <span>{label}</span>
        <small>无数据</small>
      </div>
    );
  }

  return (
    <div className="usage-meter">
      <div className="usage-meter__label">
        <span>{label}</span>
        <strong>{value}%</strong>
      </div>
      <div className="usage-meter__track">
        <div
          className={`usage-meter__fill usage-meter__fill--${usageTone(value)}`}
          style={{ width: `${value}%` }}
        />
      </div>
    </div>
  );
}

function SettingRow({
  checked,
  description,
  disabled,
  label,
  onChange,
}: {
  checked: boolean;
  description: string;
  disabled: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className={`setting-row${disabled ? " setting-row--disabled" : ""}`}>
      <div>
        <div className="setting-row__title">{label}</div>
        <div className="setting-row__description">{description}</div>
      </div>
      <input
        checked={checked}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.checked)}
        type="checkbox"
      />
    </label>
  );
}

function planTone(plan: string) {
  switch (plan.toLowerCase()) {
    case "enterprise":
    case "business":
      return "ember";
    case "team":
      return "amber";
    case "plus":
      return "ocean";
    case "pro":
      return "emerald";
    default:
      return "slate";
  }
}

function usageTone(value: number) {
  if (value >= 60) {
    return "good";
  }
  if (value >= 30) {
    return "warn";
  }
  return "danger";
}

function formatLastUsed(value: number) {
  return new Date(value * 1000).toLocaleString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    month: "numeric",
    day: "numeric",
  });
}

function toErrorMessage(error: unknown) {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "发生了未知错误";
}

export default App;
