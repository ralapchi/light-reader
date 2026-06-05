/** Schedule callback on the next animation frame. */
export function afterNextPaint(callback: () => void): number {
  return requestAnimationFrame(callback)
}

/** Schedule callback after two animation frames (layout has settled). */
export function afterLayoutSettled(callback: () => void): number {
  return requestAnimationFrame(() => {
    requestAnimationFrame(callback)
  })
}
