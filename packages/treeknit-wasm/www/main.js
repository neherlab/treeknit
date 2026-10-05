const EXAMPLE = [
  { label: 'ha', newick: '((A,B),(C,(D,X)));' },
  { label: 'na', newick: '((A,(B,X)),(C,D));' },
];

const $ = (id) => document.getElementById(id);
let trees = [];
let worker = newWorker();
let urls = [];

function newWorker() {
  return new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
}

function setTrees(list) {
  trees = list;
  $('trees').replaceChildren(...trees.map((t) => el('li', t.label)));
  $('run').disabled = trees.length < 2;
  $('results').hidden = true;
  status('');
}

function status(text, error = false) {
  $('status').textContent = text;
  $('status').classList.toggle('error', error);
}

function el(tag, text) {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = text;
  return e;
}

function download(name, text, type) {
  const a = el('a', name);
  a.href = URL.createObjectURL(new Blob([text], { type }));
  urls.push(a.href);
  a.download = name;
  return a;
}

function run(request) {
  return new Promise((resolve, reject) => {
    worker.onmessage = ({ data }) => {
      if (data.ok) return resolve(data.result);
      if (data.fatal) {
        worker.terminate();
        worker = newWorker();
      }
      reject(new Error(data.error));
    };
    worker.onerror = (e) => reject(new Error(e.message || 'the analysis could not start'));
    worker.postMessage(request);
  });
}

function show(result) {
  const pairs = result.pairs.map(({ trees: [a, b], mccs }) => {
    const table = el('table');
    table.append(el('caption', `${a} – ${b}: ${mccs.length} MCC${mccs.length === 1 ? '' : 's'}`));
    for (const [i, mcc] of mccs.entries()) {
      const row = el('tr');
      row.append(el('th', String(i + 1)), el('td', mcc.join(', ')));
      table.append(row);
    }
    return table;
  });
  $('pairs').replaceChildren(...pairs);

  const arg = result.arg;
  $('arg').textContent =
    arg === null ? '' :
    arg.status === 'built' ? `ARG: ${arg.reassortments} reassortment${arg.reassortments === 1 ? '' : 's'}` :
    `No ARG: ${arg.message}`;

  urls.forEach((u) => URL.revokeObjectURL(u));
  urls = [];
  const links = result.files.map((f) => download(f.name, f.text, f.mediaType));
  $('downloads').replaceChildren(...links);
  $('results').hidden = false;
}

$('files').addEventListener('change', async (e) => {
  const files = [...e.target.files];
  setTrees(await Promise.all(files.map(async (f) => ({
    label: f.name.replace(/\.[^.]*$/, ''),
    newick: await f.text(),
  }))));
});

$('example').addEventListener('click', () => {
  $('files').value = '';
  setTrees(EXAMPLE);
});

$('form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const settings = {
    gamma: Number($('gamma').value),
    resolve: $('resolve').value,
    seed: Number($('seed').value),
  };
  $('run').disabled = true;
  $('results').hidden = true;
  status('Running…');
  try {
    show(await run({ trees, settings }));
    status('');
  } catch (err) {
    status(err.message, true);
  } finally {
    $('run').disabled = trees.length < 2;
  }
});
