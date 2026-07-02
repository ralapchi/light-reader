const PATTERN = /\u{E000}(.+?)\u{E001}/gu

/** Return all inline image asset IDs found in text. */
export function matchInlineImages(text: string): string[] {
  const ids: string[] = []
  const re = new RegExp(PATTERN.source, PATTERN.flags)
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    ids.push(m[1])
  }
  return ids
}

/** Iterate inline image matches in text, calling fn for each. */
export function forEachInlineImage(
  text: string,
  fn: (match: RegExpExecArray) => void,
): void {
  const re = new RegExp(PATTERN.source, PATTERN.flags)
  let m: RegExpExecArray | null
  while ((m = re.exec(text)) !== null) {
    fn(m)
  }
}
