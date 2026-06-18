import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Play,
  Square,
  RefreshCcw,
  HardDrive,
  Cpu,
  MemoryStick,
  Terminal,
  Activity,
  AlertTriangle,
} from "lucide-react";
import { Header, cn } from "./OneUI";

interface SystemInfo {
  total_disk: number;
  used_disk: number;
  os_name: string;
  cpu_usage: number;
  memory_usage: number;
}

const formatSize = (bytes: number) => {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let val = bytes,
    idx = 0;
  while (val > 1024 && idx < units.length - 1) {
    val /= 1024;
    idx++;
  }
  return `${val.toFixed(1)} ${units[idx]}`;
};

export const ServerControl: React.FC = () => {
  const [isRunning, setIsRunning] = useState(false);
  const [sysInfo, setSysInfo] = useState<SystemInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);

  const checkStatus = async () => {
    try {
      setIsRunning(await invoke<boolean>("get_server_status"));
    } catch (e) {
      setStatusError(String(e));
      console.error("Failed to fetch server status:", e);
    }
  };
  const fetchSysInfo = async () => {
    try {
      setSysInfo(await invoke<SystemInfo>("get_system_info"));
    } catch (e) {
      console.error("Failed to fetch system info:", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    Promise.all([checkStatus(), fetchSysInfo()]);
    const t = setInterval(() => {
      checkStatus();
      fetchSysInfo();
    }, 3000);
    return () => clearInterval(t);
  }, []);

  const handleStart = async () => {
    setActionLoading(true);
    setError(null);
    try {
      await invoke("start_server");
      setIsRunning(true);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setActionLoading(false);
    }
  };
  const handleStop = async () => {
    setActionLoading(true);
    setError(null);
    try {
      await invoke("stop_server");
      setIsRunning(false);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setActionLoading(false);
    }
  };

  const resources = [
    {
      label: "Espace Disque",
      icon: HardDrive,
      accent: "var(--color-accent)",
      accentBg: "var(--color-accent-subtle)",
      value: sysInfo
        ? `${formatSize(sysInfo.used_disk)} / ${formatSize(sysInfo.total_disk)}`
        : "—",
      pct: sysInfo ? sysInfo.used_disk / sysInfo.total_disk : 0,
      barColor:
        sysInfo && sysInfo.used_disk / sysInfo.total_disk > 0.9
          ? "var(--color-error)"
          : "var(--color-accent)",
    },
    {
      label: "Processeur",
      icon: Cpu,
      accent: "#7a4100",
      accentBg: "var(--color-warning-bg)",
      value: sysInfo ? `${sysInfo.cpu_usage.toFixed(1)}%` : "—",
      pct: sysInfo ? sysInfo.cpu_usage / 100 : 0,
      barColor:
        sysInfo && sysInfo.cpu_usage > 80 ? "var(--color-error)" : "#7a4100",
    },
    {
      label: "Mémoire RAM",
      icon: MemoryStick,
      accent: "var(--color-success)",
      accentBg: "var(--color-success-bg)",
      value: sysInfo ? formatSize(sysInfo.memory_usage) : "—",
      pct: null,
      sub: sysInfo?.os_name,
      barColor: "var(--color-success)",
    },
  ];

  return (
    <div style={{ paddingBottom: 48 }}>
      <Header
        title="Contrôle Système"
        subtitle="Cycle de vie du serveur et surveillance des ressources"
      />

      <div style={{ padding: "0 32px 24px" }}>
        {/* Server process card */}
        <div
          className="fluent-card flex items-center justify-between"
          style={{ padding: "16px 20px" }}
        >
          <div className="flex items-center gap-4">
            <div
              style={{
                width: 40,
                height: 40,
                borderRadius: 8,
                flexShrink: 0,
                background: isRunning
                  ? "var(--color-success-bg)"
                  : "var(--color-win-nav)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Terminal
                size={20}
                style={{
                  color: isRunning
                    ? "var(--color-success)"
                    : "var(--color-win-text3)",
                }}
              />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <p style={{ fontWeight: 600, fontSize: 15, margin: 0 }}>
                  Processus Serveur
                </p>
                <span
                  className={cn(
                    "fluent-badge",
                    isRunning ? "fluent-badge-green" : "fluent-badge-red",
                  )}
                >
                  {isRunning ? "Actif" : "Inactif"}
                </span>
              </div>
              <p
                style={{
                  fontSize: 12,
                  color: "var(--color-win-text3)",
                  margin: 0,
                }}
              >
                Serveur MCP · Rust/Tauri binary
              </p>
              {statusError && (
                <p
                  style={{
                    color: "red",
                    fontSize: "0.8rem",
                    margin: "2px 0 0",
                  }}
                >
                  Status unavailable: {statusError}
                </p>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              className={cn(
                "fluent-btn flex items-center gap-2",
                isRunning ? "fluent-btn-danger" : "fluent-btn-accent",
              )}
              onClick={isRunning ? handleStop : handleStart}
              disabled={actionLoading}
              style={{ minWidth: 120 }}
            >
              {actionLoading ? (
                <RefreshCcw
                  size={14}
                  style={{ animation: "spin 0.8s linear infinite" }}
                />
              ) : isRunning ? (
                <Square size={14} fill="currentColor" />
              ) : (
                <Play size={14} fill="currentColor" />
              )}
              {isRunning ? "Arrêter" : "Démarrer"}
            </button>
            <button
              className="fluent-btn"
              style={{ padding: "6px 10px" }}
              onClick={() => {
                checkStatus();
                fetchSysInfo();
              }}
            >
              <RefreshCcw size={14} />
            </button>
          </div>
        </div>

        {/* Error */}
        {error && (
          <div
            className="flex items-center gap-3 mt-3 p-4 fluent-card"
            style={{
              borderLeft: "4px solid var(--color-error)",
              background: "var(--color-error-bg)",
            }}
          >
            <AlertTriangle
              size={18}
              style={{ color: "var(--color-error)", flexShrink: 0 }}
            />
            <div>
              <p
                style={{
                  fontWeight: 600,
                  fontSize: 14,
                  margin: 0,
                  color: "var(--color-error)",
                }}
              >
                Erreur d'exécution
              </p>
              <p
                style={{
                  fontSize: 12,
                  color: "var(--color-win-text2)",
                  margin: 0,
                }}
              >
                {error}
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Resource cards */}
      <div
        style={{
          padding: "0 28px",
          display: "grid",
          gridTemplateColumns: "repeat(3, 1fr)",
          gap: 16,
        }}
      >
        {resources.map((r, i) => {
          const Icon = r.icon;
          return (
            <div
              key={i}
              className="fluent-card"
              style={{ padding: "20px 24px" }}
            >
              <div className="flex items-center gap-3 mb-4">
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 8,
                    background: r.accentBg,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    flexShrink: 0,
                  }}
                >
                  <Icon size={18} style={{ color: r.accent }} />
                </div>
                <span
                  style={{
                    fontSize: 13,
                    color: "var(--color-win-text3)",
                    fontWeight: 400,
                  }}
                >
                  {r.label}
                </span>
              </div>
              <p
                style={{
                  fontSize: 22,
                  fontWeight: 600,
                  color: "var(--color-win-text)",
                  margin: "0 0 12px",
                  letterSpacing: "-0.3px",
                }}
              >
                {loading ? "..." : r.value}
              </p>
              {r.pct !== null && !loading && (
                <div>
                  <div className="fluent-progress">
                    <div
                      className="fluent-progress-fill"
                      style={{
                        width: `${r.pct * 100}%`,
                        background: r.barColor,
                      }}
                    />
                  </div>
                  <p
                    style={{
                      fontSize: 11,
                      color: "var(--color-win-text3)",
                      margin: "6px 0 0",
                      textAlign: "right",
                    }}
                  >
                    {(r.pct * 100).toFixed(0)}% utilisé
                  </p>
                </div>
              )}
              {r.sub && !loading && (
                <div
                  className="flex items-center gap-1.5"
                  style={{ marginTop: 8 }}
                >
                  <Activity
                    size={12}
                    style={{ color: "var(--color-win-text3)" }}
                  />
                  <span
                    style={{ fontSize: 12, color: "var(--color-win-text3)" }}
                  >
                    {r.sub}
                  </span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
