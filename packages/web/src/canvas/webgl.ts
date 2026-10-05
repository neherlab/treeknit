export const WEBGL2_MISSING = "Drawing trees needs WebGL 2, which this browser does not provide.";

export interface ProbeContext {
  getExtension(name: "WEBGL_lose_context"): { loseContext(): void } | null;
}

export interface ContextSource {
  getContext(type: "webgl2"): ProbeContext | null;
}

export function supportsWebGl2(createCanvas: () => ContextSource): boolean {
  try {
    const context = createCanvas().getContext("webgl2");

    context?.getExtension("WEBGL_lose_context")?.loseContext();

    return context !== null;
  } catch {
    return false;
  }
}

export function browserSupportsWebGl2(): boolean {
  return supportsWebGl2(() => document.createElement("canvas"));
}
