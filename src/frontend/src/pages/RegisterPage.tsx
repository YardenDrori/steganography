import { register } from "../api/auth";
import { useState } from "react";
import { tryCatch } from "../api/tryCatch";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";

function RegisterPage() {
  const [form, setForm] = useState<{
    user_name: string;
    first_name: string;
    last_name: string;
    email: string;
    phone_number: string;
    is_male: boolean | undefined;
    password: string;
  }>({
    user_name: "Xx_DickSucka892_xX",
    first_name: "Joe",
    last_name: "Schmo",
    email: "JoeSchmo@theilluminati.com",
    phone_number: "6969696969",
    is_male: undefined,
    password: "12345678",
  });
  const [confirmPassword, setConfirmPassword] = useState("12345678");
  const [error, setError] = useState<String | null>(null);
  const auth = useAuth();
  const navigate = useNavigate();

  async function handleSubmit(e: React.SubmitEvent) {
    e.preventDefault();
    if (confirmPassword === form.password) {
      const [data, caughtError] = await tryCatch(register(form));
      if (caughtError) {
        setError(caughtError);
        return;
      } else {
        console.log(data);
        auth.setAccessToken(data?.data.access_token);
        auth.setUser(data?.data.user);
        navigate("/");
      }
    } else setError("passwords don't match");
  }

  return (
    <>
      <form onSubmit={handleSubmit}>
        <p>values with '*' indicate mandatory field</p>
        <input
          type="text"
          placeholder="Username*"
          value={form.user_name}
          onChange={(e) => setForm({ ...form, user_name: e.target.value })}
        />
        <input
          type="text"
          placeholder="First name*"
          value={form.first_name}
          onChange={(e) => setForm({ ...form, first_name: e.target.value })}
        />
        <input
          type="text"
          placeholder="Last name*"
          value={form.last_name}
          onChange={(e) => setForm({ ...form, last_name: e.target.value })}
        />
        <input
          type="email"
          placeholder="Email address*"
          value={form.email}
          onChange={(e) => setForm({ ...form, email: e.target.value })}
        />
        <input
          type="tel"
          placeholder="phone number"
          value={form.phone_number}
          onChange={(e) => setForm({ ...form, phone_number: e.target.value })}
        />
        <select
          value={form.is_male === undefined ? "" : form.is_male ? "true" : "false"}
          onChange={(e) =>
            setForm({
              ...form,
              is_male:
                e.target.value === undefined ? undefined : e.target.value === "" ? true : false,
            })
          }
        >
          <option value={""}>Select gender</option>
          <option value={"true"}>Male</option>
          <option value={"false"}>Female</option>
        </select>
        <input
          type="password"
          placeholder="password*"
          value={form.password}
          onChange={(e) => setForm({ ...form, password: e.target.value })}
        />
        <input
          type="password"
          placeholder="confirm password*"
          value={confirmPassword}
          onChange={(e) => setConfirmPassword(e.target.value)}
        />
        {error && <p style={{ color: "red" }}>{error}</p>}
        <button type="submit">Sign up</button>
      </form>
      <button type="button" onClick={() => navigate("/login")}>
        Sign In
      </button>
    </>
  );
}

export default RegisterPage;
