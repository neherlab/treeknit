export function randomSeed(random: Uint32Array, max: number | null): number {
  const value = random[0] ?? 0;

  return max === null ? value : Math.min(value, max);
}

export function drawSeed(max: number | null): number {
  return randomSeed(crypto.getRandomValues(new Uint32Array(1)), max);
}
