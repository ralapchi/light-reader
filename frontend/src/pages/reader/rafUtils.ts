/** Schedule callback on the next animation frame. Returns a cancel function. */
export function afterNextPaint(callback: () => void): () => void {
  const id = requestAnimationFrame(callback)
  return () => cancelAnimationFrame(id)
}

/** Schedule callback after two animation frames (layout has settled). Returns a cancel function. */
export function afterLayoutSettled(callback: () => void): () => void {
  let outerId: number | undefined
  let innerId: number | undefined
  outerId = requestAnimationFrame(() => {
    outerId = undefined
    innerId = requestAnimationFrame(callback)
  })
  return () => {
    if (outerId !== undefined) cancelAnimationFrame(outerId)
    if (innerId !== undefined) cancelAnimationFrame(innerId)
  }
}
