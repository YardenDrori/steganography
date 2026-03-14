import { register } from "../api/auth";
import { useState } from "react";
import { tryCatch } from "../api/tryCatch";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

function RegisterPage() {
  const [form, setForm] = useState({
    user_name: "",
    first_name: "",
    last_name: "",
    email: "",
    phone_number: "",
    is_male: undefined as boolean | undefined,
    password: "",
  });
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const auth = useAuth();
  const navigate = useNavigate();

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    if (confirmPassword !== form.password) {
      setError("Passwords don't match");
      return;
    }
    const [data, caughtError] = await tryCatch(register(form));
    if (caughtError) {
      setError(String(caughtError));
    } else {
      auth.setAccessToken(data?.data.access_token);
      auth.setUser(data?.data.user);
      navigate("/");
    }
  }

  const inputClass =
    "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition";

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center px-4 py-10">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="flex flex-col items-center mb-8">
          <div className="w-12 h-12 rounded-2xl bg-indigo-600 flex items-center justify-center mb-4">
            <svg className="w-6 h-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 3.75v4.5m0-4.5h4.5m-4.5 0L9 9M3.75 20.25v-4.5m0 4.5h4.5m-4.5 0L9 15M20.25 3.75h-4.5m4.5 0v4.5m0-4.5L15 9m5.25 11.25h-4.5m4.5 0v-4.5m0 4.5L15 15" />
            </svg>
          </div>
          <h1 className="text-2xl font-bold text-white">Create an account</h1>
          <p className="text-gray-400 text-sm mt-1">Get started with Stegano</p>
        </div>

        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6">
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-gray-300 text-sm font-medium mb-1.5">First name *</label>
                <input
                  type="text"
                  placeholder="Jane"
                  value={form.first_name}
                  onChange={(e) => setForm({ ...form, first_name: e.target.value })}
                  className={inputClass}
                  required
                />
              </div>
              <div>
                <label className="block text-gray-300 text-sm font-medium mb-1.5">Last name *</label>
                <input
                  type="text"
                  placeholder="Doe"
                  value={form.last_name}
                  onChange={(e) => setForm({ ...form, last_name: e.target.value })}
                  className={inputClass}
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Username *</label>
              <input
                type="text"
                placeholder="janedoe"
                value={form.user_name}
                onChange={(e) => setForm({ ...form, user_name: e.target.value })}
                className={inputClass}
                required
              />
            </div>

            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Email *</label>
              <input
                type="email"
                placeholder="jane@example.com"
                value={form.email}
                onChange={(e) => setForm({ ...form, email: e.target.value })}
                className={inputClass}
                required
              />
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-gray-300 text-sm font-medium mb-1.5">Phone</label>
                <input
                  type="tel"
                  placeholder="+1 555 0100"
                  value={form.phone_number}
                  onChange={(e) => setForm({ ...form, phone_number: e.target.value })}
                  className={inputClass}
                />
              </div>
              <div>
                <label className="block text-gray-300 text-sm font-medium mb-1.5">Gender</label>
                <select
                  value={form.is_male === undefined ? "" : form.is_male ? "true" : "false"}
                  onChange={(e) =>
                    setForm({
                      ...form,
                      is_male:
                        e.target.value === "" ? undefined : e.target.value === "true",
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

            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Password *</label>
              <input
                type="password"
                placeholder="••••••••"
                value={form.password}
                onChange={(e) => setForm({ ...form, password: e.target.value })}
                className={inputClass}
                required
              />
            </div>

            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">Confirm password *</label>
              <input
                type="password"
                placeholder="••••••••"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className={inputClass}
                required
              />
            </div>

            {error && (
              <p className="text-red-400 text-sm bg-red-950/60 border border-red-900 rounded-lg px-4 py-3">
                {error}
              </p>
            )}

            <button
              type="submit"
              className="w-full bg-indigo-600 hover:bg-indigo-500 text-white font-semibold py-3 rounded-lg transition-colors mt-2"
            >
              Create account
            </button>
          </form>
        </div>

        <p className="text-center text-gray-500 text-sm mt-4">
          Already have an account?{" "}
          <button
            type="button"
            onClick={() => navigate("/login")}
            className="text-indigo-400 hover:text-indigo-300 font-medium transition-colors"
          >
            Sign in
          </button>
        </p>
      </div>
    </div>
  );
}

export default RegisterPage;
