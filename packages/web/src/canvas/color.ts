export type Rgba = [number, number, number, number];

const OPAQUE = 255;

const HEX_COLOR = /^#(?<digits>[\da-f]{3,4}|[\da-f]{6}|[\da-f]{8})$/iu;

const RGB_FUNCTION = /^rgba?\((?<body>[^)]*)\)$/iu;

const NUMBER = /^(?<value>[+-]?(?:\d+\.?\d*|\.\d+)(?:e[+-]?\d+)?)(?<percent>%?)$/iu;

export function parseColor(text: string): Rgba | undefined {
  const trimmed = text.trim();
  const hex = HEX_COLOR.exec(trimmed)?.groups?.["digits"];

  if (hex !== undefined) {
    return parseHex(hex);
  }

  const body = RGB_FUNCTION.exec(trimmed)?.groups?.["body"];

  return body === undefined ? undefined : parseRgbFunction(body);
}

export function withOpacity([r, g, b, a]: Rgba, opacity: number): Rgba {
  return [r, g, b, Math.round(a * clamp(opacity, 0, 1))];
}

function parseHex(digits: string): Rgba {
  const full = digits.length <= 4 ? digits.replaceAll(/./gu, "$&$&") : digits;
  const [r = 0, g = 0, b = 0, a = OPAQUE] = (full.match(/../gu) ?? []).map((pair) => Number.parseInt(pair, 16));

  return [r, g, b, a];
}

function parseRgbFunction(body: string): Rgba | undefined {
  const [colors = "", slashAlpha, extra] = body.split("/");
  const parts = colors.split(/[\s,]+/u).filter((part) => part !== "");
  const alphaText = slashAlpha ?? (parts.length === 4 ? parts.pop() : undefined);

  if (extra !== undefined || parts.length !== 3) {
    return undefined;
  }

  const [r, g, b] = parts.map(channel);
  const a = alphaText === undefined ? OPAQUE : alpha(alphaText.trim());

  if (r === undefined || g === undefined || b === undefined || a === undefined) {
    return undefined;
  }

  return [r, g, b, a];
}

function channel(text: string): number | undefined {
  const parsed = parseNumber(text);

  return parsed === undefined
    ? undefined
    : Math.round(clamp(parsed.percent ? (parsed.value / 100) * OPAQUE : parsed.value, 0, OPAQUE));
}

function alpha(text: string): number | undefined {
  const parsed = parseNumber(text);

  return parsed === undefined
    ? undefined
    : Math.round(clamp(parsed.percent ? parsed.value / 100 : parsed.value, 0, 1) * OPAQUE);
}

function parseNumber(text: string): { value: number; percent: boolean } | undefined {
  const groups = NUMBER.exec(text)?.groups;

  return groups === undefined ? undefined : { value: Number(groups["value"]), percent: groups["percent"] === "%" };
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(Math.max(value, low), high);
}
