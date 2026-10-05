export const INDICATOR_DELAY_MS = 300;

export const INDICATOR_MINIMUM_MS = 500;

export class DelayedIndicator {
  readonly #onChange: (visible: boolean) => void;
  #active = false;
  #visible = false;
  #minimumElapsed = false;
  #showTimer: ReturnType<typeof setTimeout> | undefined;
  #minimumTimer: ReturnType<typeof setTimeout> | undefined;

  constructor(onChange: (visible: boolean) => void) {
    this.#onChange = onChange;
  }

  set(active: boolean): void {
    this.#active = active;

    if (active) {
      this.#activate();
    } else {
      this.#deactivate();
    }
  }

  reset(): void {
    clearTimeout(this.#showTimer);
    clearTimeout(this.#minimumTimer);
    this.#showTimer = undefined;
    this.#minimumTimer = undefined;
    this.#active = false;
    this.#minimumElapsed = false;
    this.#change(false);
  }

  #activate(): void {
    if (this.#visible || this.#showTimer !== undefined) {
      return;
    }

    this.#showTimer = setTimeout(() => {
      this.#showTimer = undefined;
      this.#show();
    }, INDICATOR_DELAY_MS);
  }

  #deactivate(): void {
    clearTimeout(this.#showTimer);
    this.#showTimer = undefined;

    if (this.#visible && this.#minimumElapsed) {
      this.#change(false);
    }
  }

  #show(): void {
    this.#minimumElapsed = false;
    this.#change(true);
    this.#minimumTimer = setTimeout(() => {
      this.#minimumTimer = undefined;
      this.#minimumElapsed = true;

      if (!this.#active) {
        this.#change(false);
      }
    }, INDICATOR_MINIMUM_MS);
  }

  #change(visible: boolean): void {
    if (this.#visible !== visible) {
      this.#visible = visible;
      this.#onChange(visible);
    }
  }
}
