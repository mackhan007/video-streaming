/** Pull a stored (method 0) video out of a ZIP without reading the whole archive. */

const VIDEO_EXT = [".mp4", ".m4v", ".mov", ".webm", ".mkv"];
const STORE = 0;

export type ZipVideo = {
  name: string;
  method: number;
  size: number;
  localOffset: number;
};

function isVideoName(name: string): boolean {
  const lower = name.toLowerCase();
  if (lower.endsWith("/")) return false;
  return VIDEO_EXT.some((ext) => lower.endsWith(ext));
}

function basename(name: string): string {
  const parts = name.split(/[/\\]/);
  return parts[parts.length - 1] ?? name;
}

async function readEocd(file: File): Promise<{ cdSize: number; cdOffset: number }> {
  const max = Math.min(file.size, 65535 + 22);
  const buf = new Uint8Array(await file.slice(file.size - max).arrayBuffer());
  for (let i = buf.length - 22; i >= 0; i--) {
    if (buf[i] !== 0x50 || buf[i + 1] !== 0x4b) continue;
    if (buf[i + 2] !== 0x05 || buf[i + 3] !== 0x06) continue;
    const view = new DataView(buf.buffer, buf.byteOffset + i, 22);
    const comment = view.getUint16(20, true);
    if (i + 22 + comment !== buf.length) continue;
    return {
      cdSize: view.getUint32(12, true),
      cdOffset: view.getUint32(16, true),
    };
  }
  throw new Error(
    "That file is a ZIP archive, not a video. Upload an .mp4 / .webm / .mov / .mkv.",
  );
}

export async function listZipVideos(file: File): Promise<ZipVideo[]> {
  const { cdSize, cdOffset } = await readEocd(file);
  const cd = new Uint8Array(
    await file.slice(cdOffset, cdOffset + cdSize).arrayBuffer(),
  );
  const out: ZipVideo[] = [];
  let off = 0;
  while (off + 46 <= cd.length) {
    if (cd[off] !== 0x50 || cd[off + 1] !== 0x4b) break;
    const view = new DataView(cd.buffer, cd.byteOffset + off);
    const method = view.getUint16(10, true);
    const size = view.getUint32(20, true);
    const nameLen = view.getUint16(28, true);
    const extraLen = view.getUint16(30, true);
    const commentLen = view.getUint16(32, true);
    const localOffset = view.getUint32(42, true);
    const name = new TextDecoder().decode(
      cd.subarray(off + 46, off + 46 + nameLen),
    );
    off += 46 + nameLen + extraLen + commentLen;
    if (isVideoName(name)) out.push({ name, method, size, localOffset });
  }
  return out;
}

function pickZipVideo(file: File, videos: ZipVideo[]): ZipVideo {
  if (videos.length === 0) {
    throw new Error(
      "That file is a ZIP archive, not a video. Upload an .mp4 / .webm / .mov / .mkv.",
    );
  }
  const named = videos.filter((v) => basename(v.name) === file.name);
  if (named.length === 1) return named[0];
  if (videos.length === 1) return videos[0];
  throw new Error(
    `This file is a ZIP of ${videos.length} videos. Unzip it and upload one .mp4.`,
  );
}

async function sliceStoredEntry(file: File, entry: ZipVideo): Promise<File> {
  if (entry.method !== STORE) {
    throw new Error(
      "This video is inside a compressed ZIP. Unzip it and upload the .mp4.",
    );
  }
  const hdr = new Uint8Array(
    await file.slice(entry.localOffset, entry.localOffset + 30).arrayBuffer(),
  );
  const view = new DataView(hdr.buffer, hdr.byteOffset, 30);
  const nameLen = view.getUint16(26, true);
  const extraLen = view.getUint16(28, true);
  const start = entry.localOffset + 30 + nameLen + extraLen;
  const blob = file.slice(start, start + entry.size);
  const name = basename(entry.name);
  const type = name.toLowerCase().endsWith(".mov")
    ? "video/quicktime"
    : name.toLowerCase().endsWith(".webm")
      ? "video/webm"
      : name.toLowerCase().endsWith(".mkv")
        ? "video/x-matroska"
        : "video/mp4";
  return new File([blob], name, { type, lastModified: file.lastModified });
}

/** If `file` is a ZIP of stored videos, return the inner File (name match or only entry). */
export async function unwrapZipVideo(file: File): Promise<File> {
  const videos = await listZipVideos(file);
  return sliceStoredEntry(file, pickZipVideo(file, videos));
}
