import React, { useState, useEffect } from "react";
import { Header, cn, SectionDivider } from "./OneUI";
import {
  Shield,
  Save,
  RefreshCcw,
  Lock,
  Server,
  Globe,
  Zap,
  RotateCcw,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface GlobalSettingsData {
  default_storage_quota_gb: number;
  allow_public_registration: boolean;
  allow_public_links: boolean;
  server_maintenance_mode: boolean;
  max_upload_size_mb: number;
  mcp_port: number;
  webdav_port: number;
  s3_port: number;
}

const SettingRow = ({
  label,
  desc,
  checked,
  onChange,
  danger = false,
}: {
  label: string;
  desc: string;
  checked: boolean;
  onChange: () => void;
  danger?: boolean;
}) => (
  <div
    style={{
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "14px 16px",
      borderRadius: 10,
      background:
        danger && checked ? "var(--color-error-bg)" : "var(--color-win-nav)",
      border: `1px solid ${danger && checked ? "var(--color-error)" : "var(--color-win-stroke)"}`,
      marginBottom: 8,
      transition: "all 0.2s ease",
    }}
  >
    <div style={{ flex: 1, paddingRight: 16 }}>
      <p
        style={{
          fontWeight: 600,
          fontSize: 14,
          margin: 0,
          color:
            danger && checked ? "var(--color-error)" : "var(--color-win-text)",
        }}
      >
        {label}
      </p>
      <p
        style={{
          fontSize: 12,
          color: "var(--color-win-text3)",
          margin: 0,
          lineHeight: 1.4,
        }}
      >
        {desc}
      </p>
    </div>
    <div
      className={cn("fluent-toggle", checked && "on")}
      onClick={onChange}
      style={{ cursor: "pointer" }}
    />
  </div>
);

export const GlobalSettings: React.FC = () => {
  const [settings, setSettings] = useState<GlobalSettingsData | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    text: string;
    type: "success" | "error";
  } | null>(null);

  const fetchSettings = async () => {
    setLoading(true);
    try {
      const data = await invoke<GlobalSettingsData>("get_global_settings");
      setSettings(data);
    } catch (e) {
      console.error(e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSettings();
  }, []);

  const handleSave = async (overrideSettings?: GlobalSettingsData) => {
    const data = overrideSettings ?? settings;
    if (!data) return;
    setSaving(true);
    setMessage(null);
    try {
      await invoke("save_global_settings", { settings: data });
      setMessage({
        text: "Paramètres sauvegardés avec succès",
        type: "success",
      });
      setTimeout(() => setMessage(null), 3000);
    } catch (e) {
      setMessage({ text: `Erreur: ${e}`, type: "error" });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = async () => {
    if (
      !window.confirm(
        "Voulez-vous vraiment réinitialiser tous les paramètres aux valeurs par défaut ?",
      )
    )
      return;
    const defaultSettings: GlobalSettingsData = {
      default_storage_quota_gb: 5,
      allow_public_registration: true,
      allow_public_links: true,
      server_maintenance_mode: false,
      max_upload_size_mb: 500,
      mcp_port: 8081,
      webdav_port: 8083,
      s3_port: 8084,
    };
    setSettings(defaultSettings);
    // Pass defaults directly since setSettings is async and handleSave reads state
    await handleSave(defaultSettings);
  };

  if (loading && !settings)
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCcw size={24} className="animate-spin text-[--color-accent]" />
      </div>
    );

  return (
    <div style={{ paddingBottom: 60 }}>
      <Header
        title="Paramètres Système"
        subtitle="Gérez les politiques de sécurité, les quotas et les ports réseau de votre serveur Kajy."
      />

      <div
        style={{
          padding: "0 28px",
          display: "grid",
          gridTemplateColumns: "1fr 320px",
          gap: 24,
          maxWidth: 1100,
        }}
      >
        {/* Main settings column */}
        <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
          {/* Security & Access */}
          <div className="fluent-card" style={{ padding: "24px" }}>
            <div className="flex items-center gap-3 mb-6">
              <div
                style={{
                  width: 36,
                  height: 36,
                  borderRadius: 10,
                  background: "var(--color-success-bg)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <Shield size={18} style={{ color: "var(--color-success)" }} />
              </div>
              <div>
                <p style={{ fontWeight: 700, fontSize: 16, margin: 0 }}>
                  Sécurité et Accès
                </p>
                <p
                  style={{
                    fontSize: 12,
                    color: "var(--color-win-text3)",
                    margin: 0,
                  }}
                >
                  Contrôlez qui peut accéder à votre serveur
                </p>
              </div>
            </div>

            <SectionDivider label="Général" />
            <div style={{ marginTop: 12 }}>
              <SettingRow
                label="Inscriptions publiques"
                desc="Autorise n'importe qui à se créer un compte sur le serveur."
                checked={settings?.allow_public_registration ?? false}
                onChange={() =>
                  settings &&
                  setSettings({
                    ...settings,
                    allow_public_registration:
                      !settings.allow_public_registration,
                  })
                }
              />
              <SettingRow
                label="Partages externes"
                desc="Permet de générer des liens de téléchargement publics (sans compte)."
                checked={settings?.allow_public_links ?? false}
                onChange={() =>
                  settings &&
                  setSettings({
                    ...settings,
                    allow_public_links: !settings.allow_public_links,
                  })
                }
              />
              <SettingRow
                label="Mode Maintenance"
                desc="Désactive l'accès à tous les services sauf pour l'administrateur."
                checked={settings?.server_maintenance_mode ?? false}
                onChange={() =>
                  settings &&
                  setSettings({
                    ...settings,
                    server_maintenance_mode: !settings.server_maintenance_mode,
                  })
                }
                danger
              />
            </div>
          </div>

          {/* Resources & Quotas */}
          <div className="fluent-card" style={{ padding: "24px" }}>
            <div className="flex items-center gap-3 mb-6">
              <div
                style={{
                  width: 36,
                  height: 36,
                  borderRadius: 10,
                  background: "var(--color-warning-bg)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <Lock size={18} style={{ color: "var(--color-warning)" }} />
              </div>
              <div>
                <p style={{ fontWeight: 700, fontSize: 16, margin: 0 }}>
                  Ressources et Quotas
                </p>
                <p
                  style={{
                    fontSize: 12,
                    color: "var(--color-win-text3)",
                    margin: 0,
                  }}
                >
                  Limites de stockage et de transfert
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-6">
              <div className="flex flex-col gap-2">
                <label
                  style={{
                    fontSize: 12,
                    fontWeight: 700,
                    color: "var(--color-win-text2)",
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                  }}
                >
                  <Zap size={14} style={{ color: "var(--color-warning)" }} />{" "}
                  Quota par défaut (GB)
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  min="0"
                  value={settings?.default_storage_quota_gb || 0}
                  onChange={(e) =>
                    settings &&
                    setSettings({
                      ...settings,
                      default_storage_quota_gb: Math.max(
                        0,
                        Number(e.target.value),
                      ),
                    })
                  }
                />
                <p style={{ fontSize: 11, color: "var(--color-win-text4)" }}>
                  Espace alloué aux nouveaux utilisateurs.
                </p>
              </div>
              <div className="flex flex-col gap-2">
                <label
                  style={{
                    fontSize: 12,
                    fontWeight: 700,
                    color: "var(--color-win-text2)",
                    display: "flex",
                    alignItems: "center",
                    gap: 6,
                  }}
                >
                  Taille max upload (MB)
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  min="1"
                  value={settings?.max_upload_size_mb || 0}
                  onChange={(e) =>
                    settings &&
                    setSettings({
                      ...settings,
                      max_upload_size_mb: Math.max(1, Number(e.target.value)),
                    })
                  }
                />
                <p style={{ fontSize: 11, color: "var(--color-win-text4)" }}>
                  Limite par fichier lors du transfert.
                </p>
              </div>
            </div>
          </div>

          {/* Advanced Network */}
          <div className="fluent-card" style={{ padding: "24px" }}>
            <div className="flex items-center gap-3 mb-6">
              <div
                style={{
                  width: 36,
                  height: 36,
                  borderRadius: 10,
                  background: "var(--color-accent-subtle)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <Globe size={18} style={{ color: "var(--color-accent)" }} />
              </div>
              <div>
                <p style={{ fontWeight: 700, fontSize: 16, margin: 0 }}>
                  Ports et Réseau
                </p>
                <p
                  style={{
                    fontSize: 12,
                    color: "var(--color-win-text3)",
                    margin: 0,
                  }}
                >
                  Configuration des services d'écoute
                </p>
              </div>
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="flex flex-col gap-2">
                <label
                  style={{
                    fontSize: 11,
                    fontWeight: 700,
                    color: "var(--color-win-text3)",
                    textTransform: "uppercase",
                  }}
                >
                  Port MCP
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  min={1}
                  max={65535}
                  value={settings?.mcp_port || 0}
                  onChange={(e) =>
                    settings &&
                    setSettings({
                      ...settings,
                      mcp_port: Number(e.target.value),
                    })
                  }
                />
              </div>
              <div className="flex flex-col gap-2">
                <label
                  style={{
                    fontSize: 11,
                    fontWeight: 700,
                    color: "var(--color-win-text3)",
                    textTransform: "uppercase",
                  }}
                >
                  Port WebDAV
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  min={1}
                  max={65535}
                  value={settings?.webdav_port || 0}
                  onChange={(e) =>
                    settings &&
                    setSettings({
                      ...settings,
                      webdav_port: Number(e.target.value),
                    })
                  }
                />
              </div>
              <div className="flex flex-col gap-2">
                <label
                  style={{
                    fontSize: 11,
                    fontWeight: 700,
                    color: "var(--color-win-text3)",
                    textTransform: "uppercase",
                  }}
                >
                  Port S3
                </label>
                <input
                  type="number"
                  className="fluent-input"
                  min={1}
                  max={65535}
                  value={settings?.s3_port || 0}
                  onChange={(e) =>
                    settings &&
                    setSettings({
                      ...settings,
                      s3_port: Number(e.target.value),
                    })
                  }
                />
              </div>
            </div>

            <div
              style={{
                marginTop: 20,
                padding: "10px 14px",
                borderRadius: 8,
                background: "var(--color-win-bg)",
                borderLeft: "4px solid var(--color-accent)",
                display: "flex",
                alignItems: "center",
                gap: 10,
              }}
            >
              <Server size={16} style={{ color: "var(--color-accent)" }} />
              <p
                style={{
                  fontSize: 12,
                  color: "var(--color-win-text3)",
                  margin: 0,
                }}
              >
                <strong>Note:</strong> Un redémarrage manuel du serveur est
                nécessaire pour appliquer ces ports.
              </p>
            </div>
          </div>
        </div>

        {/* Action side column */}
        <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
          <div
            className="fluent-card"
            style={{
              background: "var(--color-accent)",
              border: "none",
              boxShadow: "var(--shadow-16)",
              color: "white",
              padding: "24px",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                marginBottom: 12,
              }}
            >
              <Save size={20} />
              <p style={{ fontWeight: 700, fontSize: 16, margin: 0 }}>
                Sauvegarder
              </p>
            </div>
            <p
              style={{
                fontSize: 13,
                opacity: 0.9,
                marginBottom: 20,
                lineHeight: 1.5,
              }}
            >
              Appliquer les modifications immédiatement sur le fichier de
              configuration global.
            </p>
            <button
              onClick={() => handleSave()}
              disabled={saving || !settings}
              style={{
                width: "100%",
                padding: "10px 0",
                borderRadius: 8,
                border: "1px solid rgba(255,255,255,0.2)",
                background: "rgba(255,255,255,0.15)",
                color: "white",
                fontWeight: 700,
                fontSize: 14,
                cursor: saving ? "wait" : "pointer",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                gap: 8,
                transition: "background 0.2s",
              }}
              className="hover:bg-[rgba(255,255,255,0.25)]"
            >
              {saving ? (
                <RefreshCcw size={16} className="animate-spin" />
              ) : (
                <Save size={16} />
              )}
              {saving ? "Enregistrement..." : "Enregistrer"}
            </button>

            {message && (
              <div
                style={{
                  marginTop: 16,
                  padding: "10px 12px",
                  borderRadius: 8,
                  fontSize: 13,
                  fontWeight: 600,
                  background:
                    message.type === "success"
                      ? "rgba(255,255,255,0.2)"
                      : "rgba(220,38,38,0.5)",
                  color: "white",
                  animation: "slideDown 0.3s ease-out",
                }}
              >
                {message.text}
              </div>
            )}
          </div>

          <div className="fluent-card" style={{ padding: "20px" }}>
            <div className="flex items-center gap-2 mb-4">
              <RotateCcw
                size={16}
                style={{ color: "var(--color-win-text3)" }}
              />
              <p style={{ fontWeight: 700, fontSize: 14, margin: 0 }}>
                Actions rapides
              </p>
            </div>
            <button
              className="fluent-btn w-full"
              style={{ justifyContent: "center", fontSize: 13, height: 36 }}
              onClick={handleReset}
            >
              Réinitialiser par défaut
            </button>
          </div>

          <div
            className="fluent-card"
            style={{ padding: "20px", background: "var(--color-win-nav)" }}
          >
            <p style={{ fontWeight: 700, fontSize: 13, marginBottom: 10 }}>
              Statut Config
            </p>
            <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
              <div
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: "50%",
                  background: settings ? "#107c10" : "#888",
                }}
              />
              <span style={{ fontSize: 12, color: "var(--color-win-text2)" }}>
                {settings ? "Fichier chargé" : "Erreur de chargement"}
              </span>
            </div>
            <p
              style={{
                fontSize: 11,
                color: "var(--color-win-text4)",
                marginTop: 12,
              }}
            >
              v2.1.0-stable · Build 2026.06.15
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
