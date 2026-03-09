import { useState, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { getFiles, type FileItem } from "../api/files";
import { extract, type EmbedConfigs, type Channels } from "../api/steg";
import { tryCatch } from "../api/tryCatch";

const DEFAULT_COEFFICIENTS = Array(16).fill(false);

export default function ExtractPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const { accessToken } = useAuth();

  const [files, setFiles] = useState<FileItem[]>([]);
  const [stegObjectId, setStegObjectId] = useState<number | null>(
    (location.state?.file as FileItem | undefined)?.id ?? null
  );
  const [delta, setDelta] = useState(10);
  const [coefficients, setCoefficients] = useState<boolean[]>(DEFAULT_COEFFICIENTS);
  const [channelMode, setChannelMode] = useState<"yuv" | "rgb">("yuv");
  const [yuvChannels, setYuvChannels] = useState({ y: true, cb: false, cr: false });
  const [rgbChannels, setRgbChannels] = useState({ r: true, g: false, b: false });

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);

  useEffect(() => {
    async function fetchFiles() {
      const [data, err] = await tryCatch(getFiles(accessToken!));
      if (!err) setFiles(data?.data ?? []);
    }
    fetchFiles();
  }, [accessToken]);

  function toggleCoefficient(index: number) {
    setCoefficients((prev) => prev.map((v, i) => (i === index ? !v : v)));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!stegObjectId) return;
    setError(null);
    setResult(null);

    const channels: Channels =
      channelMode === "yuv" ? { yuv: yuvChannels } : { rgb: rgbChannels };

    const configs: EmbedConfigs = {
      channels_to_embed: channels,
      coefficients_to_embed: coefficients,
      delta,
    };

    setLoading(true);
    const [data, err] = await tryCatch(
      extract(accessToken!, { steg_object_id: stegObjectId, configs })
    );
    setLoading(false);

    if (err) {
      setError(String(err));
    } else {
      setResult(`Done! Extracted file: ${data?.filename}`);
    }
  }

  const selectClass =
    "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition text-sm";

  const stegFiles = files.filter((f) => f.is_steg_object);

  return (
    <div className="min-h-screen bg-gray-950 p-6">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <button
            type="button"
            onClick={() => navigate("/my-files")}
            className="text-gray-400 hover:text-gray-200 transition-colors cursor-pointer"
          >
            ← Back
          </button>
          <h1 className="text-3xl font-bold text-white">Extract</h1>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* File selection */}
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
            <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider">File</h2>

            <div>
              <label className="block text-gray-300 text-sm mb-2">Steg object (file to extract from)</label>
              <select
                value={stegObjectId ?? ""}
                onChange={(e) => setStegObjectId(Number(e.target.value) || null)}
                className={selectClass}
              >
                <option value="">Select steg object…</option>
                {stegFiles.map((f) => (
                  <option key={f.id} value={f.id}>{f.filename}</option>
                ))}
              </select>
              {stegFiles.length === 0 && (
                <p className="text-gray-500 text-xs mt-2">No steg objects found in your files.</p>
              )}
            </div>
          </div>

          {/* Channel configuration */}
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
            <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider">Channels</h2>

            <div className="flex gap-4">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="channelMode"
                  value="yuv"
                  checked={channelMode === "yuv"}
                  onChange={() => setChannelMode("yuv")}
                  className="accent-indigo-500"
                />
                <span className="text-gray-200 text-sm">YUV</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="channelMode"
                  value="rgb"
                  checked={channelMode === "rgb"}
                  onChange={() => setChannelMode("rgb")}
                  className="accent-indigo-500"
                />
                <span className="text-gray-200 text-sm">RGB</span>
              </label>
            </div>

            {channelMode === "yuv" ? (
              <div className="flex gap-6">
                {(["y", "cb", "cr"] as const).map((ch) => (
                  <label key={ch} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={yuvChannels[ch]}
                      onChange={(e) => setYuvChannels({ ...yuvChannels, [ch]: e.target.checked })}
                      className="accent-indigo-500"
                    />
                    <span className="text-gray-200 text-sm uppercase">{ch}</span>
                  </label>
                ))}
              </div>
            ) : (
              <div className="flex gap-6">
                {(["r", "g", "b"] as const).map((ch) => (
                  <label key={ch} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={rgbChannels[ch]}
                      onChange={(e) => setRgbChannels({ ...rgbChannels, [ch]: e.target.checked })}
                      className="accent-indigo-500"
                    />
                    <span className="text-gray-200 text-sm uppercase">{ch}</span>
                  </label>
                ))}
              </div>
            )}
          </div>

          {/* DCT coefficients */}
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider">
                DCT Coefficients (4×4)
              </h2>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setCoefficients(Array(16).fill(true))}
                  className="text-xs text-indigo-400 hover:text-indigo-300 cursor-pointer"
                >
                  All
                </button>
                <span className="text-gray-600 text-xs">·</span>
                <button
                  type="button"
                  onClick={() => setCoefficients(Array(16).fill(false))}
                  className="text-xs text-gray-500 hover:text-gray-300 cursor-pointer"
                >
                  None
                </button>
              </div>
            </div>

            <div className="grid grid-cols-4 gap-2">
              {coefficients.map((val, i) => (
                <button
                  key={i}
                  type="button"
                  onClick={() => toggleCoefficient(i)}
                  className={`aspect-square rounded-lg text-xs font-medium transition-colors cursor-pointer ${
                    val
                      ? "bg-indigo-600 hover:bg-indigo-500 text-white"
                      : "bg-gray-800 hover:bg-gray-700 text-gray-500"
                  }`}
                >
                  {Math.floor(i / 4)},{i % 4}
                </button>
              ))}
            </div>
          </div>

          {/* Delta */}
          <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
            <h2 className="text-sm font-medium text-gray-400 uppercase tracking-wider">Delta</h2>
            <div className="flex items-center gap-4">
              <input
                type="range"
                min={1}
                max={100}
                value={delta}
                onChange={(e) => setDelta(Number(e.target.value))}
                className="flex-1 accent-indigo-500"
              />
              <input
                type="number"
                min={1}
                max={100}
                value={delta}
                onChange={(e) => setDelta(Number(e.target.value))}
                className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-gray-100 text-sm text-center focus:outline-none focus:ring-2 focus:ring-indigo-500"
              />
            </div>
          </div>

          {error && (
            <p className="text-red-400 text-sm bg-red-950 border border-red-900 rounded-lg px-4 py-3">
              {error}
            </p>
          )}
          {result && (
            <p className="text-green-400 text-sm bg-green-950 border border-green-900 rounded-lg px-4 py-3">
              {result}
            </p>
          )}

          <button
            type="submit"
            disabled={!stegObjectId || loading}
            className="w-full bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-700 disabled:text-gray-500 text-white font-semibold py-3 rounded-lg transition-colors cursor-pointer disabled:cursor-not-allowed"
          >
            {loading ? "Extracting…" : "Extract"}
          </button>
        </form>
      </div>
    </div>
  );
}
