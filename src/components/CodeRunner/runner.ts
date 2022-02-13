import { logToHtml } from './log-to-html';

// code to execute on a web worker for purposes of eval
const WORKER_CODE = `
const console = new Proxy({}, {
  get(_target, key) {
    return (...args) => {
      postMessage({ type: key, args });
    }
  }
});
self.addEventListener('error', e => {
  postMessage({ type: 'error', args: [e.message] });
  e.preventDefault();
});
`

interface ElementWorker {
  worker: Worker;
  terminatorId: number;
}
const workerMap = new WeakMap<HTMLPreElement, ElementWorker>();

const onExecute = (element: HTMLPreElement) => {
  if (workerMap.has(element)) {
    const { worker, terminatorId } = workerMap.get(element);

    if (worker) {
      clearTimeout(terminatorId);
      worker.terminate();
    }
  }

  const outputEl = element.querySelector('output')
  outputEl.innerHTML = '';
  outputEl.className = '';

  const code = element.querySelector('code').textContent;
  const blob = new Blob([WORKER_CODE, code], { type: 'application/javascript' });
  const url = URL.createObjectURL(blob);
  const worker = new Worker(url);

  worker.addEventListener('message', ({ data }) => {
    const { type, args } = data;

    const logEl = logToHtml(...args);
    logEl.setAttribute('data-type', type.toLowerCase());

    outputEl.appendChild(logEl);
    outputEl.classList.add('opened');

    console[type](...args);
  });

  const terminatorId = setTimeout(() => {
    worker.terminate();
  }, 5000) as unknown as number;

  workerMap.set(element, { terminatorId, worker });
}

export function addRunnableCode() {
  const runnables = document.querySelectorAll('runnable-code');

  for (const el of runnables) {
    const preTag = el.nextElementSibling;
    if (!(preTag instanceof HTMLPreElement)) {
      throw new Error('Runnable Code must preceed pre tag');
    }

    if (preTag.classList.contains('runnable-code')) continue;
    preTag.classList.add('runnable-code');

    const runButton = document.createElement('button');
    runButton.textContent = 'Run'; // '▶'
    preTag.prepend(runButton);

    const outputEl = document.createElement('output');
    preTag.append(outputEl);

    runButton.addEventListener('click', () => onExecute(preTag));
    
    el.remove();
  }
}