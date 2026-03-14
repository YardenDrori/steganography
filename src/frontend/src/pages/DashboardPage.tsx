import { useAuth } from "../context/AuthContext";
import { useNavigate } from "react-router-dom";

const features = [
  {
    to: "/my-files",
    title: "My Files",
    description: "Upload, manage and organise your carrier videos and payloads.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
      </svg>
    ),
  },
  {
    to: "/embed",
    title: "Embed",
    description: "Hide a payload inside a carrier video using DCT-based steganography.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
      </svg>
    ),
  },
  {
    to: "/extract",
    title: "Extract",
    description: "Recover a hidden payload from a steganographic video object.",
    icon: (
      <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
      </svg>
    ),
  },
];

function DashboardPage() {
  const { user } = useAuth();
  const navigate = useNavigate();

  const initials = [user?.first_name?.[0], user?.last_name?.[0]]
    .filter(Boolean)
    .join("")
    .toUpperCase() || "?";

  return (
    <div className="p-8 max-w-4xl">
      {/* Header */}
      <div className="flex items-center gap-4 mb-10">
        <div className="w-14 h-14 rounded-2xl bg-indigo-800 flex items-center justify-center flex-shrink-0">
          <span className="text-indigo-200 text-xl font-bold">{initials}</span>
        </div>
        <div>
          <h1 className="text-2xl font-bold text-white">
            Welcome back, {user?.first_name}
          </h1>
          <p className="text-gray-400 text-sm mt-0.5">@{user?.user_name}</p>
        </div>
        {!user?.is_verified && (
          <div className="ml-auto bg-amber-950/60 border border-amber-800 rounded-lg px-4 py-2.5">
            <p className="text-amber-400 text-sm font-medium">Email not verified</p>
            <p className="text-amber-600 text-xs mt-0.5">
              Go to <button onClick={() => navigate("/settings")} className="underline hover:text-amber-400">Settings</button> to verify.
            </p>
          </div>
        )}
      </div>

      {/* Feature cards */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        {features.map(({ to, title, description, icon }) => (
          <button
            key={to}
            type="button"
            onClick={() => navigate(to)}
            className="bg-gray-900 border border-gray-800 hover:border-indigo-700 rounded-2xl p-6 text-left transition-colors group"
          >
            <div className="w-10 h-10 rounded-xl bg-indigo-950 border border-indigo-900 flex items-center justify-center text-indigo-400 group-hover:bg-indigo-600 group-hover:text-white group-hover:border-indigo-600 transition-colors mb-4">
              {icon}
            </div>
            <h2 className="text-white font-semibold text-base mb-1">{title}</h2>
            <p className="text-gray-400 text-sm leading-relaxed">{description}</p>
          </button>
        ))}
      </div>

      {/* Account info */}
      <div className="mt-6 bg-gray-900 border border-gray-800 rounded-2xl p-6">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-4">Account</h2>
        <div className="grid grid-cols-2 gap-4 text-sm">
          <div>
            <p className="text-gray-500 mb-0.5">Email</p>
            <p className="text-gray-200">{user?.email}</p>
          </div>
          <div>
            <p className="text-gray-500 mb-0.5">Member since</p>
            <p className="text-gray-200">{user?.created_at?.slice(0, 10)}</p>
          </div>
          {user?.phone_number && (
            <div>
              <p className="text-gray-500 mb-0.5">Phone</p>
              <p className="text-gray-200">{user.phone_number}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default DashboardPage;
