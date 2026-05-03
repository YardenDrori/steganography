import { useState, useEffect } from "react";
import { useLocation } from "react-router-dom";
import { useAuth } from "../context/AuthContext";
import { getFiles, type FileItem } from "../api/files";
import { extract, type EmbedConfigs, type EmbedMethod, type Channels } from "../api/steg";
import { tryCatch } from "../api/tryCatch";

type PageSettings = {
  method: EmbedMethod;
  channelMode: "yuv" | "rgb";
  yuvChannels: { y: boolean; cb: boolean; cr: boolean };
  rgbChannels: { r: boolean; g: boolean; b: boolean };
  coefficients: boolean[];
  coefficientsPerBit: number;
  blocksPerMacroblock: number;
  delta: number;
  seed: string;
  parityBytes: number;
};

const DEFAULT_COEFFICIENTS = Array(16).fill(false);
const METHODS: EmbedMethod[] = ["QIM", "STDM", "SS", "ISS"];
const SEED_METHODS: EmbedMethod[] = ["STDM", "SS", "ISS"];

const selectClass =
  "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition";
const inputClass = selectClass;

export default function ExtractPage() {
  const location = useLocation();
  const { accessToken } = useAuth();

  const [files, setFiles] = useState<FileItem[]>([]);
  const [stegObjectId, setStegObjectId] = useState<number | null>(
    (location.state?.file as FileItem | undefined)?.id ?? null,
  );
  const [method, setMethod] = useState<EmbedMethod>("QIM");
  const [channelMode, setChannelMode] = useState<"yuv" | "rgb">("yuv");
  const [yuvChannels, setYuvChannels] = useState({ y: true, cb: false, cr: false });
  const [rgbChannels, setRgbChannels] = useState({ r: true, g: false, b: false });
  const [coefficients, setCoefficients] = useState<boolean[]>(DEFAULT_COEFFICIENTS);
  const [coefficientsPerBit, setCoefficientsPerBit] = useState(1);
  const [blocksPerMacroblock, setBlocksPerMacroblock] = useState(4);
  const [delta, setDelta] = useState(10);
  const [seed, setSeed] = useState("");
  const [parityBytes, setParityBytes] = useState(16);

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(null);
  const [clipboardMsg, setClipboardMsg] = useState<string | null>(null);

  useEffect(() => {
    async function fetchFiles() {
      const [data, err] = await tryCatch(getFiles(accessToken!));
      if (!err) setFiles(data?.data ?? []);
    }
    fetchFiles();
  }, [accessToken]);

  async function exportSettings() {
    const settings: PageSettings = {
      method, channelMode, yuvChannels, rgbChannels, coefficients,
      coefficientsPerBit, blocksPerMacroblock, delta, seed, parityBytes,
    };
    try {
      await navigator.clipboard.writeText(JSON.stringify(settings, null, 2));
      setClipboardMsg("Copied!");
    } catch {
      setClipboardMsg("Copy failed");
    }
    setTimeout(() => setClipboardMsg(null), 2000);
  }

  async function importSettings() {
    try {
      const text = await navigator.clipboard.readText();
      const s: PageSettings = JSON.parse(text);
      if (s.method) setMethod(s.method);
      if (s.channelMode) setChannelMode(s.channelMode);
      if (s.yuvChannels) setYuvChannels(s.yuvChannels);
      if (s.rgbChannels) setRgbChannels(s.rgbChannels);
      if (Array.isArray(s.coefficients) && s.coefficients.length === 16) setCoefficients(s.coefficients);
      if (s.coefficientsPerBit != null) setCoefficientsPerBit(s.coefficientsPerBit);
      if (s.blocksPerMacroblock != null) setBlocksPerMacroblock(s.blocksPerMacroblock);
      if (s.delta != null) setDelta(s.delta);
      if (s.seed != null) setSeed(s.seed);
      if (s.parityBytes != null) setParityBytes(s.parityBytes);
      setClipboardMsg("Imported!");
    } catch {
      setClipboardMsg("Invalid settings");
    }
    setTimeout(() => setClipboardMsg(null), 2000);
  }

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
      coefficients_per_bit: coefficientsPerBit,
      blocks_per_macroblock: blocksPerMacroblock,
      delta,
      method,
      reed_solomon_padding_byte_count: parityBytes,
      ...(SEED_METHODS.includes(method) ? { seed } : {}),
    };

    setLoading(true);
    const [data, err] = await tryCatch(
      extract(accessToken!, { steg_object_id: stegObjectId, configs }),
    );
    setLoading(false);

    if (err) {
      setError(String(err));
    } else {
      setResult(`Done! Extracted file: ${data?.filename}`);
    }
  }

  const stegFiles = files.filter((f) => f.is_steg_object);

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-8 flex items-start justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Extract</h1>
          <p className="text-gray-400 text-sm mt-1">Recover a hidden payload from a steg object.</p>
        </div>
        <div className="flex items-center gap-2 mt-1">
          {clipboardMsg && (
            <span className="text-xs text-gray-400">{clipboardMsg}</span>
          )}
          <button
            type="button"
            onClick={importSettings}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white transition-colors border border-gray-700"
          >
            Import settings
          </button>
          <button
            type="button"
            onClick={exportSettings}
            className="px-3 py-1.5 rounded-lg text-xs font-medium bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white transition-colors border border-gray-700"
          >
            Export settings
          </button>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-5">
        {/* File selection */}
        <div className="bg-gray-900 border border-gray-800 rounded-2xl p-6 space-y-4">
          <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">File</h2>
          <div>
            <label className="block text-gray-300 text-sm font-medium mb-1.5">
              Steg object
            </label>
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
              <p className="text-gray-500 text-xs mt-1.5">No steg objects found in your files.</p>
            )}
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
                placeholder="Seed used during embedding"
                value={seed}
                onChange={(e) => setSeed(e.target.value)}
                className={inputClass}
                required
              />
              <p className="text-gray-500 text-xs mt-1.5">Must match the seed that was used to embed.</p>
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

          <div>
            <label className="block text-gray-300 text-sm font-medium mb-2">
              Reed-Solomon parity bytes
            </label>
            <input
              type="number"
              min={0}
              max={247}
              value={parityBytes}
              onChange={(e) =>
                setParityBytes(Math.max(0, Math.min(247, Number(e.target.value))))
              }
              className="w-32 bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-gray-100 text-sm text-center focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-transparent transition"
            />
            <p className="text-gray-500 text-xs mt-1.5">
              Must match the parity count used during embedding.
            </p>
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
          disabled={!stegObjectId || loading || (SEED_METHODS.includes(method) && !seed.trim())}
          className="w-full bg-indigo-600 hover:bg-indigo-500 disabled:bg-gray-800 disabled:text-gray-500 text-white font-semibold py-3 rounded-lg transition-colors"
        >
          {loading ? "Extracting…" : "Extract"}
        </button>
      </form>
    </div>
  );
}
