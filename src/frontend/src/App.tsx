import { useState } from "react";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import { LoginForm } from "./components/LoginForm";
import { RegisterForm } from "./components/RegisterForm";
import { Profile } from "./components/Profile";
import { SteganographyTool } from "./components/SteganographyTool";
import { FileManager } from "./components/FileManager";
import "./App.css";

function AppContent() {
  const { user, loading } = useAuth();
  const [showRegister, setShowRegister] = useState(false);
  const [activeTab, setActiveTab] = useState<"steganography" | "files" | "profile">(
    "steganography"
  );

  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  if (!user) {
    return (
      <div className="auth-container">
        <div className="auth-header">
          <h1>Steganography App</h1>
          <p>Hide stuff in videos securely</p>
        </div>

        <div className="auth-content">
          {showRegister ? <RegisterForm /> : <LoginForm />}

          <button className="toggle-auth" onClick={() => setShowRegister(!showRegister)}>
            {showRegister ? "Already have an account? Login" : "Don't have an account? Register"}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      <header className="app-header">
        <h1>Steganography App</h1>
        <p>Welcome, {user.first_name}!</p>
      </header>

      <nav className="tabs">
        <button
          className={activeTab === "steganography" ? "active" : ""}
          onClick={() => setActiveTab("steganography")}
        >
          Steganography
        </button>
        <button
          className={activeTab === "files" ? "active" : ""}
          onClick={() => setActiveTab("files")}
        >
          Files
        </button>
        <button
          className={activeTab === "profile" ? "active" : ""}
          onClick={() => setActiveTab("profile")}
        >
          Profile
        </button>
      </nav>

      <main className="app-content">
        {activeTab === "steganography" && <SteganographyTool />}
        {activeTab === "files" && <FileManager />}
        {activeTab === "profile" && <Profile />}
      </main>
    </div>
  );
}

function App() {
  return (
    <AuthProvider>
      <AppContent />
    </AuthProvider>
  );
}

export default App;
