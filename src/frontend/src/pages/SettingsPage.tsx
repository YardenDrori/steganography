import { useState } from "react";
import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";
import { tryCatch } from "../api/tryCatch";
import { updateProfile, getSessions, revokeSession } from "../api/user";
import { changePassword, logoutAllDevices, sendVerificationEmail } from "../api/auth";
import { deleteAccount } from "../api/auth";

type Session = {
  id: number;
  device_info: string | null;
  expires_at: string;
};

function SettingsPage() {
  const { user, accessToken, setUser, setAccessToken } = useAuth();
  const navigate = useNavigate();

  const [profileForm, setProfileForm] = useState({
    first_name: user?.first_name ?? "",
    last_name: user?.last_name ?? "",
    user_name: user?.user_name ?? "",
    email: user?.email ?? "",
    phone_number: user?.phone_number ?? "",
    is_male: user?.is_male ?? undefined,
  });
  const [profileMessage, setProfileMessage] = useState<string | null>(null);

  const [passwordForm, setPasswordForm] = useState({
    old_password: "",
    new_password: "",
    confirm_password: "",
  });
  const [passwordMessage, setPasswordMessage] = useState<string | null>(null);

  const [sessions, setSessions] = useState<Session[] | null>(null);
  const [sessionsLoaded, setSessionsLoaded] = useState(false);
  const [sessionsError, setSessionsError] = useState<string | null>(null);

  async function handleProfileSave(e: React.FormEvent) {
    e.preventDefault();
    setProfileMessage(null);
    const [data, err] = await tryCatch(updateProfile(accessToken!, profileForm));
    if (err) {
      setProfileMessage("Failed: " + err);
    } else {
      setUser(data?.data);
      setProfileMessage("Profile updated");
    }
  }

  async function handlePasswordChange(e: React.FormEvent) {
    e.preventDefault();
    setPasswordMessage(null);
    if (passwordForm.new_password !== passwordForm.confirm_password) {
      setPasswordMessage("Passwords don't match");
      return;
    }
    const [, err] = await tryCatch(
      changePassword(accessToken!, passwordForm.old_password, passwordForm.new_password)
    );
    if (err) {
      setPasswordMessage("Failed: " + err);
    } else {
      setPasswordMessage("Password changed");
      setPasswordForm({ old_password: "", new_password: "", confirm_password: "" });
    }
  }

  async function handleSendVerification() {
    const [, err] = await tryCatch(sendVerificationEmail(accessToken!));
    if (err) alert("Failed to send verification email");
    else alert("Verification email sent");
  }

  async function handleLoadSessions() {
    setSessionsError(null);
    const [data, err] = await tryCatch(getSessions(accessToken!));
    if (err) {
      setSessionsError("Failed to load sessions");
    } else {
      setSessions(data?.data);
    }
    setSessionsLoaded(true);
  }

  async function handleRevokeSession(sessionId: number) {
    const [, err] = await tryCatch(revokeSession(accessToken!, sessionId));
    if (err) {
      alert("Failed to revoke session");
    } else {
      setSessions((prev) => prev?.filter((s) => s.id !== sessionId) ?? null);
    }
  }

  async function handleLogoutAll() {
    const [, err] = await tryCatch(logoutAllDevices(accessToken!));
    if (err) { alert("Failed: " + err); return; }
    setAccessToken(null);
    setUser(null);
    navigate("/login");
  }

  async function handleDeleteAccount() {
    if (!confirm("Are you sure? This will deactivate your account.")) return;
    const [, err] = await tryCatch(deleteAccount(accessToken!));
    if (err) { alert("Failed: " + err); return; }
    setAccessToken(null);
    setUser(null);
    navigate("/login");
  }

  return (
    <div>
      <h1>Settings</h1>
      <button type="button" onClick={() => navigate("/")}>
        Back to Dashboard
      </button>

      <section>
        <h2>Account Information</h2>
        <form onSubmit={handleProfileSave}>
          <input
            type="text"
            placeholder="First name"
            value={profileForm.first_name}
            onChange={(e) => setProfileForm({ ...profileForm, first_name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Last name"
            value={profileForm.last_name}
            onChange={(e) => setProfileForm({ ...profileForm, last_name: e.target.value })}
          />
          <input
            type="text"
            placeholder="Username"
            value={profileForm.user_name}
            onChange={(e) => setProfileForm({ ...profileForm, user_name: e.target.value })}
          />
          <input
            type="email"
            placeholder="Email"
            value={profileForm.email}
            onChange={(e) => setProfileForm({ ...profileForm, email: e.target.value })}
          />
          <input
            type="tel"
            placeholder="Phone number"
            value={profileForm.phone_number ?? ""}
            onChange={(e) => setProfileForm({ ...profileForm, phone_number: e.target.value })}
          />
          <select
            value={profileForm.is_male === undefined || profileForm.is_male === null ? "" : profileForm.is_male ? "true" : "false"}
            onChange={(e) =>
              setProfileForm({
                ...profileForm,
                is_male: e.target.value === "" ? undefined : e.target.value === "true",
              })
            }
          >
            <option value="">Select gender</option>
            <option value="true">Male</option>
            <option value="false">Female</option>
          </select>
          {profileMessage && <p>{profileMessage}</p>}
          <button type="submit">Save changes</button>
        </form>
      </section>

      {!user?.is_verified && (
        <section>
          <h2>Email Verification</h2>
          <p>Your email is not verified.</p>
          <button type="button" onClick={handleSendVerification}>
            Send verification email
          </button>
        </section>
      )}

      <section>
        <h2>Change Password</h2>
        <form onSubmit={handlePasswordChange}>
          <input
            type="password"
            placeholder="Current password"
            value={passwordForm.old_password}
            onChange={(e) => setPasswordForm({ ...passwordForm, old_password: e.target.value })}
          />
          <input
            type="password"
            placeholder="New password"
            value={passwordForm.new_password}
            onChange={(e) => setPasswordForm({ ...passwordForm, new_password: e.target.value })}
          />
          <input
            type="password"
            placeholder="Confirm new password"
            value={passwordForm.confirm_password}
            onChange={(e) => setPasswordForm({ ...passwordForm, confirm_password: e.target.value })}
          />
          {passwordMessage && <p>{passwordMessage}</p>}
          <button type="submit">Change password</button>
        </form>
      </section>

      <section>
        <h2>Active Sessions</h2>
        {!sessionsLoaded ? (
          <button type="button" onClick={handleLoadSessions}>
            Load sessions
          </button>
        ) : sessionsError ? (
          <p style={{ color: "red" }}>{sessionsError}</p>
        ) : (
          <ul>
            {sessions?.map((session) => (
              <li key={session.id}>
                {session.device_info ?? "Unknown device"} — expires{" "}
                {new Date(session.expires_at).toLocaleDateString()}
                <button type="button" onClick={() => handleRevokeSession(session.id)}>
                  Revoke
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section>
        <h2>Danger Zone</h2>
        <button type="button" onClick={handleLogoutAll}>
          Logout all devices
        </button>
        <button type="button" onClick={handleDeleteAccount}>
          Delete account
        </button>
      </section>
    </div>
  );
}

export default SettingsPage;
