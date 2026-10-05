const DRAW_RANGE = 2 ** 53;

const WORD_RANGE = 2 ** 32;

const HIGH_WORD_RANGE = 2 ** 21;

const UNBOUNDED_MAX = WORD_RANGE - 1;

export function randomSeed(draw: () => number, max: number | null): number {
  const count = (max ?? UNBOUNDED_MAX) + 1;
  const limit = DRAW_RANGE - (DRAW_RANGE % count);
  let value = draw();

  while (value >= limit) {
    value = draw();
  }

  return value % count;
}

export function drawSeed(max: number | null): number {
  return randomSeed(drawUniform, max);
}

function drawUniform(): number {
  const [high = 0, low = 0] = crypto.getRandomValues(new Uint32Array(2));

  return (high % HIGH_WORD_RANGE) * WORD_RANGE + low;
}
