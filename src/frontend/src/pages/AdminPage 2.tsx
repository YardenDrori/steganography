import { useState, useEffect, useCallback } from "react";
import { useAuth } from "../context/AuthContext";
import { tryCatch } from "../api/tryCatch";
import {
  getAllUsers,
  getAllFiles,
  activateUser,
  deactivateUser,
  deleteUserAdmin,
  checkServiceHealth,
  type AdminUser,
  type AdminFile,
  type ServiceHealth,
} from "../api/admin";
import { deleteFile } from "../api/files";

type Tab = "overview" | "users" | "files" | "health";

function formatDate(iso: string) {
  return new Date(iso).toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

function Badge({ active }: { active: boolean }) {
  return (
    <span
      className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border ${
        active
          ? "bg-green-950/60 text-green-400 border-green-900"
          : "bg-red-950/60 text-red-400 border-red-900"
      }`}
    >
      {active ? "Active" : "Inactive"}
    </span>
  );
}

function FileTypeBadge({ isCarrier, isSteg }: { isCarrier: boolean; isSteg: boolean }) {
  if (isCarrier)
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-indigo-950/60 text-indigo-400 border-indigo-900">
        Carrier
      </span>
    );
  if (isSteg)
    return (
      <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-yellow-950/60 text-yellow-400 border-yellow-900">
        Stego
      </span>
    );
  return (
    <span className="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium border bg-gray-800 text-gray-400 border-gray-700">
      Regular
    </span>
  );
}

function ConfirmModal({
  message,
  onConfirm,
  onCancel,
}: {
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 max-w-sm w-full mx-4 shadow-xl">
        <p className="text-gray-200 text-sm mb-6">{message}</p>
        <div className="flex gap-3 justify-end">
          <button
            onClick={onCancel}
            className="px-4 py-2 rounded-lg text-sm font-medium bg-gray-800 hover:bg-gray-700 border border-gray-700 text-gray-200 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="px-4 py-2 rounded-lg text-sm font-medium bg-red-950/60 hover:bg-red-900/60 border border-red-900 text-red-400 transition-colors"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}

export default function AdminPage() {
  const { accessToken } = useAuth();
  const [tab, setTab] = useState<Tab>("overview");

  const [users, setUsers] = useState<AdminUser[]>([]);
  const [files, setFiles] = useState<AdminFile[]>([]);
  const [health, setHealth] = useState<ServiceHealth[]>([]);
  const [healthLoading, setHealthLoading] = useState(false);

  const [loadingUsers, setLoadingUsers] = useState(false);
  const [loadingFiles, setLoadingFiles] = useState(false);

  const [userSearch, setUserSearch] = useState("");
  const [fileSearch, setFileSearch] = useState("");

  const [confirm, setConfirm] = useState<{ message: string; onConfirm: () => void } | null>(null);
  const [togglingUser, setTogglingUser] = useState<number | null>(null);

  const fetchUsers = useCallback(async () => {
    if (!accessToken) return;
    setLoadingUsers(true);
    const [res] = await tryCatch(getAllUsers(accessToken));
    if (res) setUsers(res.data);
    setLoadingUsers(false);
  }, [accessToken]);

  const fetchFiles = useCallback(async () => {
    if (!accessToken) return;
    setLoadingFiles(true);
    const [res] = await tryCatch(getAllFiles(accessToken));
    if (res) setFiles(res.data);
    setLoadingFiles(false);
  }, [accessToken]);

  const fetchHealth = useCallback(async () => {
    if (!accessToken) return;
    setHealthLoading(true);
    const results = await checkServiceHealth(accessToken);
    setHealth(results);
    setHealthLoading(false);
  }, [accessToken]);

  useEffect(() => {
    fetchUsers();
    fetchFiles();
  }, [fetchUsers, fetchFiles]);

  useEffect(() => {
    if (tab === "health" && health.length === 0) fetchHealth();
  }, [tab, health.length, fetchHealth]);

  async function handleToggleActive(user: AdminUser) {
    if (!accessToken) return;
    setTogglingUser(user.id);
    if (user.is_active) {
      await tryCatch(deactivateUser(accessToken, user.id));
    } else {
      await tryCatch(activateUser(accessToken, user.id));
    }
    await fetchUsers();
    setTogglingUser(null);
  }

  function promptDeleteUser(user: AdminUser) {
    setConfirm({
      message: `Delete user "${user.user_name}"? This cannot be undone.`,
      onConfirm: async () => {
        setConfirm(null);
        if (!accessToken) return;
        await tryCatch(deleteUserAdmin(accessToken, user.id));
        await fetchUsers();
      },
    });
  }

  function promptDeleteFile(file: AdminFile) {
    setConfirm({
      message: `Delete file "${file.filename}"? This cannot be undone.`,
      onConfirm: async () => {
        setConfirm(null);
        if (!accessToken) return;
        await tryCatch(deleteFile(accessToken, file.id));
        await fetchFiles();
      },
    });
  }

  const filteredUsers = users.filter(
    (u) =>
      u.user_name.toLowerCase().includes(userSearch.toLowerCase()) ||
      u.email.toLowerCase().includes(userSearch.toLowerCase()) ||
      `${u.first_name} ${u.last_name}`.toLowerCase().includes(userSearch.toLowerCase())
  );

  const filteredFiles = files.filter((f) =>
    f.filename.toLowerCase().includes(fileSearch.toLowerCase())
  );

  const activeUsers = users.filter((u) => u.is_active).length;
  const carrierFiles = files.filter((f) => f.is_carrier).length;
  const stegoFiles = files.filter((f) => f.is_steg_object).length;

  const tabs: { id: Tab; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "users", label: "Users" },
    { id: "files", label: "Files" },
    { id: "health", label: "System Health" },
  ];

  return (
    <div className="p-8 max-w-6xl">
      {confirm && (
        <ConfirmModal
          message={confirm.message}
          onConfirm={confirm.onConfirm}
          onCancel={() => setConfirm(null)}
        />
      )}

      <div className="mb-6">
        <h1 className="text-2xl font-bold text-white">Admin Panel</h1>
        <p className="text-gray-400 text-sm mt-1">Manage users, files, and monitor system health.</p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 mb-6 bg-gray-900 border border-gray-800 rounded-xl p-1 w-fit">
        {tabs.map(({ id, label }) => (
          <button
            key={id}
            onClick={() => setTab(id)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              tab === id
                ? "bg-indigo-600 text-white"
                : "text-gray-400 hover:text-gray-100 hover:bg-gray-800"
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Overview */}
      {tab === "overview" && (
        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
          {[
            { label: "Total Users", value: users.length },
            { label: "Active Users", value: activeUsers },
            { label: "Total Files", value: files.length },
            { label: "Carrier Files", value: carrierFiles },
            { label: "Stego Files", value: stegoFiles },
          ].map(({ label, value }) => (
            <div
              key={label}
              className="bg-gray-900 border border-gray-800 rounded-2xl p-5 flex flex-col gap-1"
            >
              <span className="text-gray-400 text-xs font-medium">{label}</span>
              <span className="text-white text-3xl font-bold">{value}</span>
            </div>
          ))}
        </div>
      )}

      {/* Users */}
      {tab === "users" && (
        <div className="bg-gray-900 border border-gray-800 rounded-2xl overflow-hidden">
          <div className="p-4 border-b border-gray-800 flex items-center justify-between gap-4">
            <input
              type="text"
              placeholder="Search users..."
              value={userSearch}
              onChange={(e) => setUserSearch(e.target.value)}
              className="bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition w-64"
            />
            <span className="text-gray-500 text-xs">{filteredUsers.length} users</span>
          </div>
          {loadingUsers ? (
            <div className="p-8 text-center text-gray-500 text-sm">Loading...</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-800">
                    {["ID", "Username", "Full Name", "Email", "Status", "Joined", "Actions"].map(
                      (h) => (
                        <th
                          key={h}
                          className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                        >
                          {h}
                        </th>
                      )
                    )}
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-800">
                  {filteredUsers.map((user) => (
                    <tr key={user.id} className="hover:bg-gray-800/50 transition-colors">
                      <td className="px-4 py-3 text-gray-500 font-mono text-xs">{user.id}</td>
                      <td className="px-4 py-3 text-gray-200 font-medium">{user.user_name}</td>
                      <td className="px-4 py-3 text-gray-300">
                        {user.first_name} {user.last_name}
                      </td>
                      <td className="px-4 py-3 text-gray-400">{user.email}</td>
                      <td className="px-4 py-3">
                        <Badge active={user.is_active} />
                      </td>
                      <td className="px-4 py-3 text-gray-500 text-xs">{formatDate(user.created_at)}</td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <button
                            onClick={() => handleToggleActive(user)}
                            disabled={togglingUser === user.id}
                            className={`px-3 py-1 rounded-lg text-xs font-medium border transition-colors disabled:opacity-50 ${
                              user.is_active
                                ? "bg-yellow-950/60 hover:bg-yellow-900/60 border-yellow-900 text-yellow-400"
                                : "bg-green-950/60 hover:bg-green-900/60 border-green-900 text-green-400"
                            }`}
                          >
                            {togglingUser === user.id
                              ? "..."
                              : user.is_active
                              ? "Deactivate"
                              : "Activate"}
                          </button>
                          <button
                            onClick={() => promptDeleteUser(user)}
                            className="px-3 py-1 rounded-lg text-xs font-medium border bg-red-950/60 hover:bg-red-900/60 border-red-900 text-red-400 transition-colors"
                          >
                            Delete
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {filteredUsers.length === 0 && (
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
        </div>
      )}

      {/* Files */}
      {tab === "files" && (
        <div className="bg-gray-900 border border-gray-800 rounded-2xl overflow-hidden">
          <div className="p-4 border-b border-gray-800 flex items-center justify-between gap-4">
            <input
              type="text"
              placeholder="Search files..."
              value={fileSearch}
              onChange={(e) => setFileSearch(e.target.value)}
              className="bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition w-64"
            />
            <span className="text-gray-500 text-xs">{filteredFiles.length} files</span>
          </div>
          {loadingFiles ? (
            <div className="p-8 text-center text-gray-500 text-sm">Loading...</div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-gray-800">
                    {["ID", "Filename", "Owner ID", "Type", "Uploaded", "Actions"].map((h) => (
                      <th
                        key={h}
                        className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                      >
                        {h}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody className="divide-y divide-gray-800">
                  {filteredFiles.map((file) => (
                    <tr key={file.id} className="hover:bg-gray-800/50 transition-colors">
                      <td className="px-4 py-3 text-gray-500 font-mono text-xs">{file.id}</td>
                      <td className="px-4 py-3 text-gray-200 font-medium max-w-xs truncate">
                        {file.filename}
                      </td>
                      <td className="px-4 py-3 text-gray-400 font-mono text-xs">{file.user_id}</td>
                      <td className="px-4 py-3">
                        <FileTypeBadge isCarrier={file.is_carrier} isSteg={file.is_steg_object} />
                      </td>
                      <td className="px-4 py-3 text-gray-500 text-xs">
                        {formatDate(file.created_at)}
                      </td>
                      <td className="px-4 py-3">
                        <button
                          onClick={() => promptDeleteFile(file)}
                          className="px-3 py-1 rounded-lg text-xs font-medium border bg-red-950/60 hover:bg-red-900/60 border-red-900 text-red-400 transition-colors"
                        >
                          Delete
                        </button>
                      </td>
                    </tr>
                  ))}
                  {filteredFiles.length === 0 && (
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
      )}

      {/* System Health */}
      {tab === "health" && (
        <div>
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-gray-300 text-sm font-medium">Service Status</h2>
            <button
              onClick={fetchHealth}
              disabled={healthLoading}
              className="px-4 py-2 rounded-lg text-sm font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition-colors disabled:opacity-50"
            >
              {healthLoading ? "Checking..." : "Refresh"}
            </button>
          </div>
          {healthLoading && health.length === 0 ? (
            <div className="text-center text-gray-500 text-sm py-8">Checking services...</div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              {health.map(({ name, ok }) => (
                <div
                  key={name}
                  className={`bg-gray-900 border rounded-2xl p-6 flex items-center justify-between ${
                    ok ? "border-green-900" : "border-red-900"
                  }`}
                >
                  <div>
                    <p className="text-white font-medium text-sm">{name} Service</p>
                    <p className={`text-xs mt-1 ${ok ? "text-green-400" : "text-red-400"}`}>
                      {ok ? "Online" : "Unreachable"}
                    </p>
                  </div>
                  <div
                    className={`w-3 h-3 rounded-full ${ok ? "bg-green-400" : "bg-red-400"}`}
                  />
                </div>
              ))}
              {health.length === 0 && (
                <div className="col-span-3 text-center text-gray-500 text-sm py-8">
                  Click Refresh to check service health.
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
