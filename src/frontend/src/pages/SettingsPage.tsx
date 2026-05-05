import { useState } from "react";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";
import { tryCatch } from "../api/tryCatch";
import { updateProfile, getSessions, revokeSession } from "../api/user";
import { changePassword, logoutAllDevices, deleteAccount } from "../api/auth";

type Session = {
  id: number;
  device_info: string | null;
  expires_at: string;
};

const inputClass =
  "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition";

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
      <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">{title}</h2>
      {children}
    </div>
  );
}

function StatusMsg({ msg, isError }: { msg: string; isError?: boolean }) {
  if (!msg) return null;
  return (
    <p className={`text-sm rounded-lg px-4 py-3 border ${
      isError
        ? "text-red-400 bg-red-950/60 border-red-900"
        : "text-green-400 bg-green-950/60 border-green-900"
    }`}>
      {msg}
    </p>
  );
}

function SettingsPage() {
  const { user, accessToken, setUser, setAccessToken } = useAuth();
  const navigate = useNavigate();

  const [profileForm, setProfileForm] = useState({
    first_name: user?.first_name ?? "",
    last_name: user?.last_name ?? "",
    user_name: user?.user_name ?? "",
    email: user?.email ?? "",
    phone_number: user?.phone_number ?? "",
    is_male: user?.is_male ?? undefined as boolean | undefined,
  });
  const [profileMsg, setProfileMsg] = useState("");
  const [profileError, setProfileError] = useState(false);

  const [passwordForm, setPasswordForm] = useState({
    old_password: "",
    new_password: "",
    confirm_password: "",
  });
  const [passwordMsg, setPasswordMsg] = useState("");
  const [passwordError, setPasswordError] = useState(false);

  const [sessions, setSessions] = useState<Session[] | null>(null);
  const [sessionsLoaded, setSessionsLoaded] = useState(false);
  const [sessionsError, setSessionsError] = useState<string | null>(null);

  async function handleProfileSave(e: React.FormEvent) {
    e.preventDefault();
    setProfileMsg("");
    const [data, err] = await tryCatch(updateProfile(accessToken!, profileForm));
    if (err) {
      setProfileMsg("Failed to update profile");
      setProfileError(true);
    } else {
      setUser({ ...data?.data, is_admin: user?.is_admin ?? false });
      setProfileMsg("Profile updated successfully");
      setProfileError(false);
    }
  }

  async function handlePasswordChange(e: React.FormEvent) {
    e.preventDefault();
    setPasswordMsg("");
    if (passwordForm.new_password !== passwordForm.confirm_password) {
      setPasswordMsg("Passwords don't match");
      setPasswordError(true);
      return;
    }
    const [, err] = await tryCatch(
      changePassword(accessToken!, passwordForm.old_password, passwordForm.new_password),
    );
    if (err) {
      setPasswordMsg("Failed to change password");
      setPasswordError(true);
    } else {
      setPasswordMsg("Password changed successfully");
      setPasswordError(false);
      setPasswordForm({ old_password: "", new_password: "", confirm_password: "" });
    }
  }

  async function handleLoadSessions() {
    setSessionsError(null);
    const [data, err] = await tryCatch(getSessions(accessToken!));
    if (err) setSessionsError("Failed to load sessions");
    else setSessions(data?.data);
    setSessionsLoaded(true);
  }

  async function handleRevokeSession(sessionId: number) {
    const [, err] = await tryCatch(revokeSession(accessToken!, sessionId));
    if (!err) setSessions((prev) => prev?.filter((s) => s.id !== sessionId) ?? null);
  }

  async function handleLogoutAll() {
    const [, err] = await tryCatch(logoutAllDevices(accessToken!));
    if (err) return;
    setAccessToken(null);
    setUser(null);
    navigate("/login");
  }

  async function handleDeleteAccount() {
    if (!confirm("Are you sure? This action cannot be undone.")) return;
    const [, err] = await tryCatch(deleteAccount(accessToken!));
    if (err) return;
    setAccessToken(null);
    setUser(null);
    navigate("/login");
  }

  return (
    <div className="p-8 max-w-2xl space-y-5">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 text-sm mt-1">Manage your account preferences.</p>
      </div>

      {/* Profile */}
      <Section title="Account Information">
        <form onSubmit={handleProfileSave} className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">First name</label>
              <input type="text" value={profileForm.first_name}
                onChange={(e) => setProfileForm({ ...profileForm, first_name: e.target.value })}
                className={inputClass} />
            </div>
            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Last name</label>
              <input type="text" value={profileForm.last_name}
                onChange={(e) => setProfileForm({ ...profileForm, last_name: e.target.value })}
                className={inputClass} />
            </div>
          </div>
          <div>
            <label className="block text-gray-300 text-sm font-medium mb-1.5">Username</label>
            <input type="text" value={profileForm.user_name}
              onChange={(e) => setProfileForm({ ...profileForm, user_name: e.target.value })}
              className={inputClass} />
          </div>
          <div>
            <label className="block text-gray-300 text-sm font-medium mb-1.5">Email</label>
            <input type="email" value={profileForm.email}
              onChange={(e) => setProfileForm({ ...profileForm, email: e.target.value })}
              className={inputClass} />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Phone</label>
              <input type="tel" value={profileForm.phone_number ?? ""}
                onChange={(e) => setProfileForm({ ...profileForm, phone_number: e.target.value })}
                className={inputClass} />
            </div>
            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Gender</label>
              <select
                value={profileForm.is_male == null ? "" : profileForm.is_male ? "true" : "false"}
                onChange={(e) =>
                  setProfileForm({
                    ...profileForm,
                    is_male: e.target.value === "" ? undefined : e.target.value === "true",
                  })
                }
                className={inputClass}
              >
                <option value="">Prefer not to say</option>
                <option value="true">Male</option>
                <option value="false">Female</option>
              </select>
            </div>
          </div>
          <StatusMsg msg={profileMsg} isError={profileError} />
          <button type="submit" className="bg-indigo-600 hover:bg-indigo-500 text-white font-semibold px-5 py-2.5 rounded-lg transition-colors text-sm">
            Save changes
          </button>
        </form>
      </Section>

      {/* Change password */}
      <Section title="Change Password">
        <form onSubmit={handlePasswordChange} className="space-y-3">
          {(
            [
              { key: "old_password", label: "Current password" },
              { key: "new_password", label: "New password" },
              { key: "confirm_password", label: "Confirm new password" },
            ] as const
          ).map(({ key, label }) => (
            <div key={key}>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">{label}</label>
              <input
                type="password"
                placeholder="••••••••"
                value={passwordForm[key]}
                onChange={(e) => setPasswordForm({ ...passwordForm, [key]: e.target.value })}
                className={inputClass}
              />
            </div>
          ))}
          <StatusMsg msg={passwordMsg} isError={passwordError} />
          <button type="submit" className="bg-indigo-600 hover:bg-indigo-500 text-white font-semibold px-5 py-2.5 rounded-lg transition-colors text-sm">
            Change password
          </button>
        </form>
      </Section>

      {/* Sessions */}
      <Section title="Active Sessions">
        {!sessionsLoaded ? (
          <button
            type="button"
            onClick={handleLoadSessions}
            className="bg-gray-800 hover:bg-gray-700 text-gray-200 font-medium px-4 py-2.5 rounded-lg transition-colors text-sm border border-gray-700"
          >
            Load sessions
          </button>
        ) : sessionsError ? (
          <p className="text-red-400 text-sm">{sessionsError}</p>
        ) : sessions?.length === 0 ? (
          <p className="text-gray-400 text-sm">No active sessions.</p>
        ) : (
          <ul className="space-y-2">
            {sessions?.map((session) => (
              <li key={session.id} className="flex items-center justify-between bg-gray-800 rounded-lg px-4 py-3">
                <div>
                  <p className="text-gray-200 text-sm">{session.device_info ?? "Unknown device"}</p>
                  <p className="text-gray-500 text-xs mt-0.5">
                    Expires {new Date(session.expires_at).toLocaleDateString()}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => handleRevokeSession(session.id)}
                  className="text-red-400 hover:text-red-300 text-sm font-medium px-3 py-1.5 rounded hover:bg-red-950 transition-colors"
                >
                  Revoke
                </button>
              </li>
            ))}
          </ul>
        )}
      </Section>

      {/* Danger zone */}
      <Section title="Danger Zone">
        <p className="text-gray-400 text-sm">These actions are permanent and cannot be undone.</p>
        <div className="flex flex-wrap gap-3 pt-1">
          <button
            type="button"
            onClick={handleLogoutAll}
            className="bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium px-4 py-2.5 rounded-lg transition-colors text-sm border border-gray-700"
          >
            Logout all devices
          </button>
          <button
            type="button"
            onClick={handleDeleteAccount}
            className="bg-red-950/60 hover:bg-red-900/60 text-red-400 font-medium px-4 py-2.5 rounded-lg transition-colors text-sm border border-red-900"
          >
            Delete account
          </button>
        </div>
      </Section>
    </div>
  );
}

export default SettingsPage;
