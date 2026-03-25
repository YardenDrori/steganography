import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { getFiles, type FileItem } from "../api/files";
import { embed, type EmbedConfigs, type EmbedMethod, type Channels } from "../api/steg";
import { tryCatch } from "../api/tryCatch";

const DEFAULT_COEFFICIENTS = Array(16).fill(false);
const METHODS: EmbedMethod[] = ["QIM", "STDM", "SS", "ISS"];
const SEED_METHODS: EmbedMethod[] = ["STDM", "SS", "ISS"];

const inputClass =
  "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition";
const selectClass = inputClass;

export default function EmbedPage() {
  const location = useLocation();
  const { accessToken } = useAuth();

  const [files, setFiles] = useState<FileItem[]>([]);
  const [carrierId, setCarrierId] = useState<number | null>(
    (location.state?.carrier as FileItem | undefined)?.id ?? null,
  );
  const [payloadId, setPayloadId] = useState<number | null>(null);
  const [method, setMethod] = useState<EmbedMethod>("QIM");
  const [channelMode, setChannelMode] = useState<"yuv" | "rgb">("yuv");
  const [yuvChannels, setYuvChannels] = useState({ y: true, cb: false, cr: false });
  const [rgbChannels, setRgbChannels] = useState({ r: true, g: false, b: false });
  const [coefficients, setCoefficients] = useState<boolean[]>(DEFAULT_COEFFICIENTS);
  const [coefficientsPerBit, setCoefficientsPerBit] = useState(1);
  const [blocksPerMacroblock, setBlocksPerMacroblock] = useState(4);
  const [delta, setDelta] = useState(10);
  const [seed, setSeed] = useState("");

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
    if (!carrierId || !payloadId) return;
    setError(null);
    setResult(null);

    const channels: Channels =
      channelMode === "yuv" ? { yuv: yuvChannels } : { rgb: rgbChannels };

    const configs: EmbedConfigs = {
      channels_to_embed: channels,
      coefficients_to_embed: coefficients,
      coefficients_per_bit: coefficientsPerBit,
      blocks_per_macroblock: blocksPerMacroblock,
      delta,
      method,
      ...(SEED_METHODS.includes(method) ? { seed } : {}),
    };

    setLoading(true);
    const [data, err] = await tryCatch(
      embed(accessToken!, { carrier_id: carrierId, payload_id: payloadId, configs }),
    );
    setLoading(false);

    if (err) {
      setError(String(err));
    } else {
      setResult(`Done! Created steg object: ${data?.filename}`);
    }
  }

  const carrierFiles = files.filter((f) => f.is_carrier);

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-white">Embed</h1>
        <p className="text-gray-400 text-sm mt-1">Hide a payload inside a carrier video.</p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-5">
        {/* Files */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Files</h2>

          <div>
            <label className="block text-gray-300 text-sm font-medium mb-1.5">
              Carrier video
            </label>
            <select
              value={carrierId ?? ""}
              onChange={(e) => setCarrierId(Number(e.target.value) || null)}
              className={selectClass}
            >
              <option value="">Select carrier video…</option>
              {carrierFiles.map((f) => (
                <option key={f.id} value={f.id}>{f.filename}</option>
              ))}
            </select>
            {carrierFiles.length === 0 && (
              <p className="text-gray-500 text-xs mt-1.5">No carrier videos in your files.</p>
            )}
          </div>

          <div>
            <label className="block text-gray-300 text-sm font-medium mb-1.5">
              Payload (file to hide)
            </label>
            <select
              value={payloadId ?? ""}
              onChange={(e) => setPayloadId(Number(e.target.value) || null)}
              className={selectClass}
            >
              <option value="">Select payload file…</option>
              {files.map((f) => (
                <option key={f.id} value={f.id}>{f.filename}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Method */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Method</h2>
          <div className="grid grid-cols-4 gap-2">
            {METHODS.map((m) => (
              <button
                key={m}
                type="button"
                onClick={() => setMethod(m)}
                className={`py-2.5 rounded-lg text-sm font-medium transition-colors ${
                  method === m
                    ? "bg-indigo-600 text-white"
                    : "bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-gray-200"
                }`}
              >
                {m}
              </button>
            ))}
          </div>

          {SEED_METHODS.includes(method) && (
            <div>
              <label className="block text-gray-300 text-sm font-medium mb-1.5">
                Seed
              </label>
              <input
                type="text"
                placeholder="Seed used for embedding"
                value={seed}
                onChange={(e) => setSeed(e.target.value)}
                className={inputClass}
                required
              />
            </div>
          )}
        </div>

        {/* Channels */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Channels</h2>

          <div className="flex gap-4">
            {(["yuv", "rgb"] as const).map((mode) => (
              <label key={mode} className="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="channelMode"
                  value={mode}
                  checked={channelMode === mode}
                  onChange={() => setChannelMode(mode)}
                  className="accent-indigo-500"
                />
                <span className="text-gray-200 text-sm uppercase">{mode}</span>
              </label>
            ))}
          </div>

          <div className="flex gap-6">
            {channelMode === "yuv"
              ? (["y", "cb", "cr"] as const).map((ch) => (
                  <label key={ch} className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={yuvChannels[ch]}
                      onChange={(e) => setYuvChannels({ ...yuvChannels, [ch]: e.target.checked })}
                      className="accent-indigo-500"
                    />
                    <span className="text-gray-200 text-sm uppercase">{ch}</span>
                  </label>
                ))
              : (["r", "g", "b"] as const).map((ch) => (
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
        </div>

        {/* DCT Coefficients */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">
              DCT Coefficients (4×4)
            </h2>
            <div className="flex gap-3">
              <button
                type="button"
                onClick={() => setCoefficients(Array(16).fill(true))}
                className="text-xs text-indigo-400 hover:text-indigo-300"
              >
                All
              </button>
              <button
                type="button"
                onClick={() => setCoefficients(Array(16).fill(false))}
                className="text-xs text-gray-500 hover:text-gray-300"
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
                className={`aspect-square rounded-lg text-xs font-medium transition-colors ${
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

        {/* Parameters */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-5">
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Parameters</h2>

          <div>
            <label className="block text-gray-300 text-sm font-medium mb-2">
              Coefficients per bit
            </label>
            <input
              type="number"
              min={1}
              max={255}
              value={coefficientsPerBit}
              onChange={(e) => setCoefficientsPerBit(Math.max(1, Number(e.target.value)))}
              className="w-32 bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 text-sm text-center focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
            />
          </div>

          <div>
            <label className="block text-gray-300 text-sm font-medium mb-2">
              Block size
            </label>
            <select
              value={blocksPerMacroblock}
              onChange={(e) => setBlocksPerMacroblock(Number(e.target.value))}
              className="w-48 bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
            >
              <option value={1}>4×4 (max capacity)</option>
              <option value={2}>8×8</option>
              <option value={4}>16×16 (max robustness)</option>
            </select>
          </div>

          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-gray-300 text-sm font-medium">Delta</label>
              <span className="text-gray-400 text-sm font-mono">{delta}</span>
            </div>
            <div className="flex items-center gap-4">
              <input
                type="range"
                min={1}
                max={255}
                value={delta}
                onChange={(e) => setDelta(Number(e.target.value))}
                className="flex-1 accent-indigo-500"
              />
              <input
                type="number"
                min={1}
                max={255}
                value={delta}
                onChange={(e) => setDelta(Number(e.target.value))}
                className="w-20 bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-gray-100 text-sm text-center focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
              />
            </div>
          </div>
        </div>

        {error && (
          <p className="text-red-400 text-sm bg-red-950/60 border border-red-900 rounded-lg px-4 py-3">
            {error}
          </p>
        )}
        {result && (
          <p className="text-green-400 text-sm bg-green-950/60 border border-green-900 rounded-lg px-4 py-3">
            {result}
          </p>
        )}

        <button
          type="submit"
          disabled={!carrierId || !payloadId || loading || (SEED_METHODS.includes(method) && !seed.trim())}
          className="w-full bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-800 disabled:text-gray-500 text-white font-semibold py-3 rounded-lg transition-colors"
        >
          {loading ? "Embedding…" : "Embed"}
        </button>
      </form>
    </div>
  );
}
