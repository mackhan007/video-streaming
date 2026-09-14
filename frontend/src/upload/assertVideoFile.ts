/** Reject non-video containers before EMS create (ZIP disguised as .mp4, etc.). */
export async function assertVideoFile(file: File): Promise<void> {
  const buf = new Uint8Array(await file.slice(0, 16).arrayBuffer());
  if (buf.length < 4) {
    throw new Error("File is empty or unreadable");
  }
  // ZIP
  if (buf[0] === 0x50 && buf[1] === 0x4b) {
    throw new Error(
      "That file is a ZIP archive, not a video. Upload an .mp4 / .webm / .mov / .mkv.",
    );
  }
  // ISO BMFF (mp4 / mov): ....ftyp
  if (
    buf.length >= 8 &&
    buf[4] === 0x66 &&
    buf[5] === 0x74 &&
    buf[6] === 0x79 &&
    buf[7] === 0x70
  ) {
    return;
  }
  // Matroska / WebM EBML
  if (
    buf[0] === 0x1a &&
    buf[1] === 0x45 &&
    buf[2] === 0xdf &&
    buf[3] === 0xa3
  ) {
    return;
  }
  throw new Error(
    "File does not look like a video (need mp4 / webm / mov / mkv).",
  );
}
