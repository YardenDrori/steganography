import { useEffect, useState, useCallback, useMemo } from "react";
import { useAuth } from "../context/AuthContext";
import { tryCatch } from "../api/tryCatch";
import { downloadFile } from "../api/files";
import {
  listAllUsers,
  listAllFiles,
  activateUser,
  deactivateUser,
  adminDeleteUser,
  adminDeleteFile,
} from "../api/admin";

type AdminUser = {
  id: number;
  user_name: string;
  first_name: string;
  last_name: string;
  email: string;
  is_active: boolean;
  created_at: string;
};

type AdminFile = {
  id: number;
  user_id: number;
  filename: string;
  is_carrier: boolean;
  is_steg_object: boolean;
  created_at: string;
};

type Tab = "overview" | "users" | "files";

export default function AdminPage() {
  const { accessToken } = useAuth();
  const [tab, setTab] = useState<Tab>("overview");
  const [users, setUsers] = useState<AdminUser[]>([]);
  const [files, setFiles] = useState<AdminFile[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<number | null>(null);

  const fetchData = useCallback(async () => {
    if (!accessToken) return;
    setLoading(true);
    setError(null);
    const [[usersRes, usersErr], [filesRes, filesErr]] = await Promise.all([
      tryCatch(listAllUsers(accessToken)),
      tryCatch(listAllFiles(accessToken)),
    ]);
    if (usersErr || filesErr) {
      setError("Failed to load admin data.");
    } else {
      setUsers(usersRes?.data ?? []);
      setFiles(filesRes?.data ?? []);
    }
    setLoading(false);
  }, [accessToken]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  async function handleToggleActive(user: AdminUser) {
    if (!accessToken) return;
    setActionLoading(user.id);
    const action = user.is_active ? deactivateUser : activateUser;
    const [, err] = await tryCatch(action(accessToken, user.id));
    if (!err) {
      setUsers((prev) =>
        prev.map((u) => (u.id === user.id ? { ...u, is_active: !u.is_active } : u))
      );
    }
    setActionLoading(null);
  }

  async function handleDeleteUser(id: number) {
    if (!accessToken || !confirm("Delete this user permanently?")) return;
    setActionLoading(id);
    const [, err] = await tryCatch(adminDeleteUser(accessToken, id));
    if (!err) setUsers((prev) => prev.filter((u) => u.id !== id));
    setActionLoading(null);
  }

  async function handleDeleteFile(id: number) {
    if (!accessToken || !confirm("Delete this file permanently?")) return;
    setActionLoading(id);
    const [, err] = await tryCatch(adminDeleteFile(accessToken, id));
    if (!err) setFiles((prev) => prev.filter((f) => f.id !== id));
    setActionLoading(null);
  }

  async function handleDownloadFile(file: AdminFile) {
    if (!accessToken) return;
    setActionLoading(file.id);
    await tryCatch(downloadFile(accessToken, file.id, file.filename));
    setActionLoading(null);
  }

  const userMap = useMemo(
    () => new Map(users.map((u) => [u.id, u.user_name])),
    [users]
  );

  const activeUsers = users.filter((u) => u.is_active).length;
  const inactiveUsers = users.length - activeUsers;
  const carrierFiles = files.filter((f) => f.is_carrier).length;
  const stegFiles = files.filter((f) => f.is_steg_object).length;

  const tabClass = (t: Tab) =>
    `px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
      tab === t
        ? "bg-indigo-600 text-white"
        : "text-gray-400 hover:text-gray-100 hover:bg-gray-800"
    }`;

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full min-h-screen">
        <p className="text-gray-400 text-sm">Loading…</p>
      </div>
    );
  }

  return (
    <div className="p-8 max-w-7xl mx-auto">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white">Admin Panel</h1>
        <p className="text-gray-400 text-sm mt-1">Manage users, files, and platform statistics.</p>
      </div>

      {error && (
        <div className="mb-6 px-4 py-3 rounded-lg bg-red-950/50 border border-red-800 text-red-400 text-sm">
          {error}
        </div>
      )}

      {/* Tabs */}
      <div className="flex gap-2 mb-8">
        <button className={tabClass("overview")} onClick={() => setTab("overview")}>Overview</button>
        <button className={tabClass("users")} onClick={() => setTab("users")}>
          Users <span className="ml-1 text-xs opacity-70">({users.length})</span>
        </button>
        <button className={tabClass("files")} onClick={() => setTab("files")}>
          Files <span className="ml-1 text-xs opacity-70">({files.length})</span>
        </button>
      </div>

      {/* Overview */}
      {tab === "overview" && (
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
          {[
            { label: "Total Users", value: users.length, color: "indigo" },
            { label: "Active Users", value: activeUsers, color: "green" },
            { label: "Inactive Users", value: inactiveUsers, color: "red" },
            { label: "Total Files", value: files.length, color: "indigo" },
            { label: "Carrier Files", value: carrierFiles, color: "purple" },
            { label: "Steg Files", value: stegFiles, color: "yellow" },
          ].map(({ label, value, color }) => (
            <div
              key={label}
              className="bg-gray-900 border border-gray-800 rounded-xl p-6"
            >
              <p className="text-gray-400 text-sm mb-2">{label}</p>
              <p
                className={`text-3xl font-bold text-${color}-400`}
              >
                {value}
              </p>
            </div>
          ))}
        </div>
      )}

      {/* Users */}
      {tab === "users" && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800 text-gray-400 text-left">
                <th className="px-4 py-3 font-medium">ID</th>
                <th className="px-4 py-3 font-medium">Username</th>
                <th className="px-4 py-3 font-medium">Name</th>
                <th className="px-4 py-3 font-medium">Email</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Registered</th>
                <th className="px-4 py-3 font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {users.map((user) => (
                <tr key={user.id} className="border-b border-gray-800/50 hover:bg-gray-800/30">
                  <td className="px-4 py-3 text-gray-500">{user.id}</td>
                  <td className="px-4 py-3 text-gray-200 font-mono text-xs">{user.user_name}</td>
                  <td className="px-4 py-3 text-gray-200">
                    {user.first_name} {user.last_name}
                  </td>
                  <td className="px-4 py-3 text-gray-400">{user.email}</td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${
                        user.is_active
                          ? "bg-green-950/60 text-green-400 border border-green-800"
                          : "bg-red-950/60 text-red-400 border border-red-800"
                      }`}
                    >
                      {user.is_active ? "Active" : "Inactive"}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-gray-500 text-xs">
                    {new Date(user.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleToggleActive(user)}
                        disabled={actionLoading === user.id}
                        className={`px-2.5 py-1 rounded-md text-xs font-medium transition-colors disabled:opacity-50 ${
                          user.is_active
                            ? "bg-yellow-900/40 text-yellow-400 hover:bg-yellow-900/70 border border-yellow-800/50"
                            : "bg-green-900/40 text-green-400 hover:bg-green-900/70 border border-green-800/50"
                        }`}
                      >
                        {actionLoading === user.id
                          ? "…"
                          : user.is_active
                          ? "Deactivate"
                          : "Activate"}
                      </button>
                      <button
                        onClick={() => handleDeleteUser(user.id)}
                        disabled={actionLoading === user.id}
                        className="px-2.5 py-1 rounded-md text-xs font-medium bg-red-900/40 text-red-400 hover:bg-red-900/70 border border-red-800/50 transition-colors disabled:opacity-50"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {users.length === 0 && (
                <tr>
                  <td colSpan={7} className="px-4 py-8 text-center text-gray-500 text-sm">
                    No users found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* Files */}
      {tab === "files" && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800 text-gray-400 text-left">
                <th className="px-4 py-3 font-medium">ID</th>
                <th className="px-4 py-3 font-medium">Filename</th>
                <th className="px-4 py-3 font-medium">Owner</th>
                <th className="px-4 py-3 font-medium">Type</th>
                <th className="px-4 py-3 font-medium">Uploaded</th>
                <th className="px-4 py-3 font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {files.map((file) => {
                const type = file.is_steg_object
                  ? "Steg"
                  : file.is_carrier
                  ? "Carrier"
                  : "Plain";
                const typeStyle =
                  file.is_steg_object
                    ? "bg-yellow-950/60 text-yellow-400 border-yellow-800"
                    : file.is_carrier
                    ? "bg-purple-950/60 text-purple-400 border-purple-800"
                    : "bg-gray-800 text-gray-400 border-gray-700";
                return (
                  <tr key={file.id} className="border-b border-gray-800/50 hover:bg-gray-800/30">
                    <td className="px-4 py-3 text-gray-500">{file.id}</td>
                    <td className="px-4 py-3 text-gray-200 font-mono text-xs max-w-xs truncate">
                      {file.filename}
                    </td>
                    <td className="px-4 py-3 text-gray-400 font-mono text-xs">
                      {userMap.get(file.user_id) ?? `#${file.user_id}`}
                    </td>
                    <td className="px-4 py-3">
                      <span
                        className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border ${typeStyle}`}
                      >
                        {type}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-gray-500 text-xs">
                      {new Date(file.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleDownloadFile(file)}
                          disabled={actionLoading === file.id}
                          className="px-2.5 py-1 rounded-md text-xs font-medium bg-indigo-900/40 text-indigo-400 hover:bg-indigo-900/70 border border-indigo-800/50 transition-colors disabled:opacity-50"
                        >
                          {actionLoading === file.id ? "…" : "Download"}
                        </button>
                        <button
                          onClick={() => handleDeleteFile(file.id)}
                          disabled={actionLoading === file.id}
                          className="px-2.5 py-1 rounded-md text-xs font-medium bg-red-900/40 text-red-400 hover:bg-red-900/70 border border-red-800/50 transition-colors disabled:opacity-50"
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
              {files.length === 0 && (
                <tr>
                  <td colSpan={6} className="px-4 py-8 text-center text-gray-500 text-sm">
                    No files found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
