import { unwrapZipVideo } from "./unwrapZipVideo";

/** Reject non-video containers; unwrap a stored video inside a ZIP named .mp4. */

const ISO_BOXES = new Set(["ftyp", "mdat", "moov", "free", "skip", "wide", "pnot"]);

function boxType(buf: Uint8Array, offset: number): string {
  if (buf.length < offset + 4) return "";
  return String.fromCharCode(
    buf[offset],
    buf[offset + 1],
    buf[offset + 2],
    buf[offset + 3],
  );
}

function isIsoBmff(buf: Uint8Array): boolean {
  return buf.length >= 8 && ISO_BOXES.has(boxType(buf, 4));
}

function isEbml(buf: Uint8Array): boolean {
  return (
    buf.length >= 4 &&
    buf[0] === 0x1a &&
    buf[1] === 0x45 &&
    buf[2] === 0xdf &&
    buf[3] === 0xa3
  );
}

function isZipMagic(buf: Uint8Array): boolean {
  return (
    buf.length >= 4 &&
    buf[0] === 0x50 &&
    buf[1] === 0x4b &&
    (buf[2] === 0x03 || buf[2] === 0x05 || buf[2] === 0x07)
  );
}

export async function assertVideoFile(
  file: File,
  depth = 0,
): Promise<File> {
  const buf = new Uint8Array(await file.slice(0, 32).arrayBuffer());
  if (buf.length < 4) {
    throw new Error("File is empty or unreadable");
  }
  // ISO BMFF first: a ~1GB mdat box size is 0x50 0x4B … and looks like "PK".
  if (isIsoBmff(buf) || isEbml(buf)) return file;
  if (isZipMagic(buf)) {
    if (depth > 0) {
      throw new Error(
        "That file is a ZIP archive, not a video. Upload an .mp4 / .webm / .mov / .mkv.",
      );
    }
    return assertVideoFile(await unwrapZipVideo(file), depth + 1);
  }
  throw new Error(
    "File does not look like a video (need mp4 / webm / mov / mkv).",
  );
}
