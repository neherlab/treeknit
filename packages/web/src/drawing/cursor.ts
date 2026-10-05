export function drawingCursor({ isDragging, isHovering }: { isDragging: boolean; isHovering: boolean }): string {
  if (isDragging) {
    return "grabbing";
  }

  return isHovering ? "pointer" : "grab";
}
