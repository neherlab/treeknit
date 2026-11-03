export declare function htmlToSvg(root: SvgSourceElement): SvgMarkup;

export interface SvgSourceElement {
  readonly nodeType: number;
  readonly tagName: string;
  readonly childNodes: Iterable<SvgSourceElement | SvgSourceText>;
  getBoundingClientRect(): SvgSourceRect;
}

export interface SvgSourceText {
  readonly nodeType: number;
  readonly nodeValue: string;
  readonly parentElement: { closest(selector: string): { getAttribute(name: string): string | null } | null };
}

export interface SvgSourceRect {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
}

export interface SvgMarkup {
  width: number;
  height: number;
  markup: string;
}
