import { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { getFiles, uploadFile, downloadFile, deleteFile, renameFile } from "../api/files";
import { tryCatch } from "../api/tryCatch";
import type { FileItem } from "../api/files";

function FileBadge({ label, color }: { label: string; color: string }) {
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${color}`}>
      {label}
    </span>
  );
}

function MyFilesPage() {
  const { accessToken } = useAuth();
  const navigate = useNavigate();
  const [files, setFiles] = useState<FileItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);
  const [renamingId, setRenamingId] = useState<number | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function fetchFiles() {
    const [data, err] = await tryCatch(getFiles(accessToken!));
    if (err) setError(String(err));
    else setFiles(data?.data ?? []);
    setIsLoading(false);
  }

  useEffect(() => {
    fetchFiles();
  }, [accessToken]);

  async function handleUpload(file: File) {
    setUploading(true);
    setError(null);
    const [, err] = await tryCatch(uploadFile(accessToken!, file));
    if (err) setError(String(err));
    else await fetchFiles();
    setUploading(false);
  }

  async function handleDelete(id: number) {
    const [, err] = await tryCatch(deleteFile(accessToken!, id));
    if (err) setError(String(err));
    else setFiles((prev) => prev.filter((f) => f.id !== id));
  }

  async function handleRename(id: number) {
    if (!renameValue.trim()) return;
    const [, err] = await tryCatch(renameFile(accessToken!, id, { new_name: renameValue }));
    if (err) {
      setError(String(err));
    } else {
      setFiles((prev) => prev.map((f) => (f.id === id ? { ...f, filename: renameValue } : f)));
      setRenamingId(null);
      setRenameValue("");
    }
  }

  async function handleDownload(file: FileItem) {
    const [, err] = await tryCatch(downloadFile(accessToken!, file.id, file.filename));
    if (err) setError(String(err));
  }

  return (
    <div className="p-8 max-w-4xl">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-white">My Files</h1>
          <p className="text-gray-400 text-sm mt-1">{files.length} file{files.length !== 1 ? "s" : ""}</p>
        </div>
        <div className="flex gap-3">
          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) handleUpload(f);
              e.target.value = "";
            }}
          />
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={uploading}
            className="flex items-center gap-2 bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-700 disabled:text-gray-500 text-white font-semibold px-4 py-2.5 rounded-lg transition-colors text-sm"
          >
            {uploading ? (
              <>
                <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z" />
                </svg>
                Uploading…
              </>
            ) : (
              <>
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                </svg>
                Upload
              </>
            )}
          </button>
        </div>
      </div>

      {error && (
        <div className="mb-6 text-red-400 text-sm bg-red-950/60 border border-red-900 rounded-lg px-4 py-3">
          {error}
        </div>
      )}

      {isLoading ? (
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-12 text-center">
          <p className="text-gray-400 text-sm">Loading files…</p>
        </div>
      ) : files.length === 0 ? (
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-12 text-center">
          <div className="w-12 h-12 rounded-xl bg-gray-800 flex items-center justify-center mx-auto mb-3">
            <svg className="w-6 h-6 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
            </svg>
          </div>
          <p className="text-gray-400 font-medium">No files yet</p>
          <p className="text-gray-500 text-sm mt-1">Upload a file to get started.</p>
        </div>
      ) : (
        <div className="bg-gray-900 border border-gray-800 rounded-2xl overflow-hidden">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="text-left text-xs font-semibold text-gray-400 uppercase tracking-wider px-6 py-3.5">Name</th>
                <th className="text-left text-xs font-semibold text-gray-400 uppercase tracking-wider px-4 py-3.5">Type</th>
                <th className="text-left text-xs font-semibold text-gray-400 uppercase tracking-wider px-4 py-3.5">Date</th>
                <th className="px-4 py-3.5" />
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-800">
              {files.map((file) => (
                <tr key={file.id} className="hover:bg-gray-800/50 transition-colors group">
                  <td className="px-6 py-4">
                    {renamingId === file.id ? (
                      <div className="flex items-center gap-2">
                        <input
                          value={renameValue}
                          onChange={(e) => setRenameValue(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") handleRename(file.id);
                            if (e.key === "Escape") setRenamingId(null);
                          }}
                          autoFocus
                          className="bg-gray-800 border border-indigo-600 rounded px-3 py-1.5 text-gray-100 text-sm focus:outline-none focus:ring-1 focus:ring-indigo-500"
                        />
                        <button
                          onClick={() => handleRename(file.id)}
                          className="text-indigo-400 hover:text-indigo-300 text-xs font-medium"
                        >
                          Save
                        </button>
                        <button
                          onClick={() => setRenamingId(null)}
                          className="text-gray-500 hover:text-gray-300 text-xs"
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <span className="text-gray-200 font-medium">{file.filename}</span>
                    )}
                  </td>
                  <td className="px-4 py-4">
                    <div className="flex gap-1.5 flex-wrap">
                      {file.is_carrier && (
                        <FileBadge label="carrier" color="bg-blue-950 text-blue-400 border border-blue-900" />
                      )}
                      {file.is_steg_object && (
                        <FileBadge label="steg object" color="bg-purple-950 text-purple-400 border border-purple-900" />
                      )}
                      {!file.is_carrier && !file.is_steg_object && (
                        <FileBadge label="payload" color="bg-gray-800 text-gray-400 border border-gray-700" />
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-4 text-gray-500">{file.created_at.slice(0, 10)}</td>
                  <td className="px-4 py-4">
                    <div className="flex items-center gap-1 justify-end opacity-0 group-hover:opacity-100 transition-opacity">
                      {file.is_carrier && (
                        <button
                          onClick={() => navigate("/embed", { state: { carrier: file } })}
                          className="text-xs text-indigo-400 hover:text-indigo-300 px-2 py-1 rounded hover:bg-indigo-950 transition-colors"
                          title="Embed into this carrier"
                        >
                          Embed
                        </button>
                      )}
                      {file.is_steg_object && (
                        <button
                          onClick={() => navigate("/extract", { state: { file } })}
                          className="text-xs text-purple-400 hover:text-purple-300 px-2 py-1 rounded hover:bg-purple-950 transition-colors"
                          title="Extract from this steg object"
                        >
                          Extract
                        </button>
                      )}
                      <button
                        onClick={() => handleDownload(file)}
                        className="text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-800 transition-colors"
                      >
                        Download
                      </button>
                      <button
                        onClick={() => { setRenamingId(file.id); setRenameValue(file.filename); }}
                        className="text-xs text-gray-400 hover:text-gray-200 px-2 py-1 rounded hover:bg-gray-800 transition-colors"
                      >
                        Rename
                      </button>
                      <button
                        onClick={() => handleDelete(file.id)}
                        className="text-xs text-red-500 hover:text-red-400 px-2 py-1 rounded hover:bg-red-950 transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

export default MyFilesPage;
