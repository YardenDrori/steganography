import { useState, useEffect } from "react";
import { useAuth } from "../context/AuthContext";
import { getFiles, uploadFile, downloadFile, deleteFile, renameFile } from "../api/files";
import { tryCatch } from "../api/tryCatch";
import type { FileItem } from "../api/files";

function MyFilesPage() {
  const { accessToken } = useAuth();
  const [files, setFiles] = useState<FileItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [renamingId, setRenamingId] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");

  useEffect(() => {
    fetchFiles();
  }, [accessToken]);

  async function fetchFiles() {
    const [data, err] = await tryCatch(getFiles(accessToken!));
    if (err) setError(err);
    else setFiles(data?.data ?? []);
    setIsLoading(false);
  }

  async function handleUpload() {
    if (!selectedFile) return;
    setUploading(true);
    setError(null);
    const [, err] = await tryCatch(uploadFile(accessToken!, selectedFile));
    if (err) setError(err);
    else await fetchFiles();
    setUploading(false);
    setSelectedFile(null);
  }

  async function handleDelete(id: number) {
    const [, err] = await tryCatch(deleteFile(accessToken!, id));
    if (err) setError(err);
    else setFiles((prev) => prev.filter((f) => f.id !== id));
  }

  async function handleRename(id: number) {
    if (!renameValue.trim()) return;
    const [, err] = await tryCatch(renameFile(accessToken!, id, { new_name: renameValue }));
    if (err) {
      setError(err);
    } else {
      setFiles((prev) => prev.map((f) => f.id === id ? { ...f, filename: renameValue } : f));
      setRenamingId(null);
      setRenameValue("");
    }
  }

  async function handleDownload(file: FileItem) {
    const [, err] = await tryCatch(downloadFile(accessToken!, file.id, file.filename));
    if (err) setError(err);
  }

  return (
    <div style={{ maxWidth: 600, margin: "40px auto", fontFamily: "sans-serif" }}>
      <h1>My Files</h1>

      <div style={{ display: "flex", gap: 8, marginBottom: 24 }}>
        <input type="file" onChange={(e) => setSelectedFile(e.target.files?.[0] ?? null)} />
        <button onClick={handleUpload} disabled={!selectedFile || uploading}>
          {uploading ? "Uploading..." : "Upload"}
        </button>
      </div>

      {error && <p style={{ color: "red" }}>{error}</p>}
      {isLoading && <p>Loading...</p>}

      {files.length === 0 && !isLoading && <p>No files yet.</p>}

      <ul style={{ listStyle: "none", padding: 0 }}>
        {files.map((file) => (
          <li key={file.id} style={{ borderBottom: "1px solid #ccc", padding: "12px 0", display: "flex", alignItems: "center", gap: 8 }}>
            {renamingId === file.id ? (
              <>
                <input value={renameValue} onChange={(e) => setRenameValue(e.target.value)} autoFocus />
                <button onClick={() => handleRename(file.id)}>Save</button>
                <button onClick={() => setRenamingId(null)}>Cancel</button>
              </>
            ) : (
              <>
                <span style={{ flex: 1 }}>{file.filename}</span>
                <small style={{ color: "#888" }}>{file.created_at.slice(0, 10)}</small>
                <button onClick={() => handleDownload(file)}>Download</button>
                <button onClick={() => { setRenamingId(file.id); setRenameValue(file.filename); }}>Rename</button>
                <button onClick={() => handleDelete(file.id)} style={{ color: "red" }}>Delete</button>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default MyFilesPage;
