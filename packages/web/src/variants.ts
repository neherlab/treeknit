export function everyVariantOf<T extends string>() {
  return <const V extends readonly [T, ...T[]]>(
    values: V & ([Exclude<T, V[number]>] extends [never] ? unknown : never),
  ) => values;
}
