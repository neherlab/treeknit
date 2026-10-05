import init, { analyze } from './pkg/treeknit_web.js';

const ready = init();

self.onmessage = async ({ data: request }) => {
  try {
    await ready;
    self.postMessage({ ok: true, result: analyze(request) });
  } catch (e) {
    // A panic or trap leaves the WebAssembly instance unusable.
    self.postMessage({ ok: false, error: String(e.message ?? e), fatal: e instanceof WebAssembly.RuntimeError });
  }
};
